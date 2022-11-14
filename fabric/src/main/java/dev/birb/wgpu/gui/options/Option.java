package dev.birb.wgpu.gui.options;

import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParseException;
import dev.birb.wgpu.gui.OptionPages;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Supplier;

public abstract class Option<T> {
	public final Text name, tooltip;
	public final boolean requiresRestart;

	private final Supplier<T> getter;
	private final Consumer<T> setter;

	private T value;

	Option(Text name, Text tooltip, boolean requiresRestart, Supplier<T> getter, Consumer<T> setter) {
		this.name = name;
		this.tooltip = tooltip;
		this.requiresRestart = requiresRestart;
		this.getter = getter;
		this.setter = setter;

		value = getter.get();
	}

	public T get() {
		return value;
	}

	public void set(T value) {
		this.value = value;
	}

	public boolean isChanged() {
		return !value.equals(getter.get());
	}

	public void apply() {
		if (isChanged()) setter.accept(value);
	}

	public void undo() {
		value = getter.get();
	}

	public abstract Widget createWidget(int x, int y, int width);

	public Text getName() {
		if (isChanged()) {
			MutableText name = this.name.copy();
			name.append(" *").formatted(Formatting.ITALIC);
			return name;
		}

		return name;
	}

	@SuppressWarnings("unchecked")
	public abstract static class Builder<B, T> {
		protected Text name, tooltip;
		protected boolean requiresRestart;
		protected Supplier<T> getter;
		protected Consumer<T> setter;

		public B setName(MutableText name) {
			this.name = name;
			return (B) this;
		}

		public B setTooltip(Text tooltip, boolean requiresRestart) {
			this.tooltip = tooltip;
			this.requiresRestart = requiresRestart;
			return (B) this;
		}

		public B setTooltip(Text tooltip) {
			return setTooltip(tooltip, false);
		}

		public B setAccessors(Supplier<T> getter, Consumer<T> setter) {
			this.getter = getter;
			this.setter = setter;
			return (B) this;
		}

		public abstract Option<T> build();
	}

	/**
	 * This needs to be implemented for List<Option> because we need to get
	 */
	public static class OptionDeserializer implements JsonDeserializer<List<Option<?>>> {
		private static Option<?> parse_option(JsonObject jsonObject, String name)
				throws JsonParseException, IllegalStateException {
			var structure = OptionPages.SETTINGS_STRUCTURE.get(name);
			var type = jsonObject.get("type");
			var typePrimitive = type.getAsJsonPrimitive();
			String typeString = typePrimitive.getAsString();
			switch (typeString) {
				case "bool" -> {
					boolean value = jsonObject.get("value").getAsJsonPrimitive().getAsBoolean();
					return new BoolOption(Text.of(name), Text.of(structure.desc()), structure.needsRestart(),
							() -> value, (bool) -> {
					});
				}
//				case "string" -> {
//					String value = jsonObject.get("value").getAsJsonPrimitive().getAsString();
//					// TODO: StringOption
//				}
//				case "float" -> {
//					double value = jsonObject.get("value").getAsJsonPrimitive().getAsDouble();
//					double min = null;
//					var min_primitive = jsonObject.get("min").getAsJsonPrimitive();
//					if ()
//					// TODO: FloatOption
//				}
				case "int" -> {
					int value = jsonObject.get("value").getAsJsonPrimitive().getAsInt();
					int min = jsonObject.get("min").getAsJsonPrimitive().getAsInt();
					int max = jsonObject.get("max").getAsJsonPrimitive().getAsInt();
					int step = jsonObject.get("step").getAsJsonPrimitive().getAsInt();

					return new IntOption(Text.of(name), Text.of(structure.desc()), structure.needsRestart(), () -> value,
							(i) -> {
							}, min, max, step, IntOption.STANDARD_FORMATTER);
				}
//				case "enum" -> {
//
//				}
				default -> throw new JsonParseException("Unexpected value: " + typeString);
			}
		}

		@Override
		public List<Option<?>> deserialize(JsonElement json, Type typeOfT, JsonDeserializationContext context)
				throws JsonParseException {
			if (json instanceof JsonObject jsonObject) {
				var options = new ArrayList<Option<?>>();
				for (var entry : jsonObject.entrySet()) {
					try {
						options.add(parse_option(entry.getValue().getAsJsonObject(), entry.getKey()));
					} catch (IllegalStateException e) {
						throw new JsonParseException(e);
					}
				}
				return options;
			} else {
				throw new JsonParseException("Tried to deserialize to List<Option<?>>, found a json element that's not an option");
			}
		}
	}
}
