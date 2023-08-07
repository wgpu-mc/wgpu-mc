package dev.birb.wgpu.gui.options;

import com.google.gson.*;
import dev.birb.wgpu.gui.OptionPages;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.client.option.SimpleOption;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Supplier;

public abstract class Option<T> {
    public final Text name;
    public final Text tooltip;
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
            return name.copy().append(" *").formatted(Formatting.ITALIC);
        }

        return name;
    }

    @SuppressWarnings("unchecked")
    public abstract static class Builder<B extends Builder<B, T>, T> {
        protected Text name;
        protected Text tooltip;
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

        // Simple wrapper around minecraft 1.19's SimpleOption, to be reflected on how to handle for wgpu-mc's config
        public B setOption(SimpleOption<T> option, Consumer<T> callback) {
            this.getter = option::getValue;
            this.setter = v -> {
                option.setValue(v);
                callback.accept(v);
            };

            return (B) this;
        }

        public B setOption(SimpleOption<T> option) {
            this.getter = option::getValue;
            this.setter = option::setValue;
            return (B) this;
        }

        public abstract Option<T> build();
    }

    public static class OptionSerializerDeserializer implements JsonDeserializer<List<Option<?>>>, JsonSerializer<List<Option<?>>> {

        private static Option<?> deserializeOption(JsonObject jsonObject, String name) throws JsonParseException, IllegalStateException {
            var structure = OptionPages.SETTINGS_STRUCTURE.get(name);
            var type = jsonObject.get("type");
            var typePrimitive = type.getAsJsonPrimitive();
            String typeString = typePrimitive.getAsString();
            switch (typeString) {
                case "bool" -> {
                    boolean value = jsonObject.get("value").getAsJsonPrimitive().getAsBoolean();
                    return new BoolOption(Text.of(name), Text.of(structure.getDesc()), structure.isNeedsRestart(), () -> value, bool -> {
                    });
                }
                case "float" -> {
                    double value = jsonObject.get("value").getAsJsonPrimitive().getAsDouble();
                    double min = jsonObject.get("min").getAsJsonPrimitive().getAsDouble();
                    double max = jsonObject.get("max").getAsJsonPrimitive().getAsDouble();
                    double step = jsonObject.get("step").getAsJsonPrimitive().getAsDouble();

                    return new FloatOption(Text.of(name), Text.of(structure.getDesc()), structure.isNeedsRestart(), () -> value, i -> {
                    }, min, max, step, FloatOption.STANDARD_FORMATTER);
                }
                case "int" -> {
                    int value = jsonObject.get("value").getAsJsonPrimitive().getAsInt();
                    int min = jsonObject.get("min").getAsJsonPrimitive().getAsInt();
                    int max = jsonObject.get("max").getAsJsonPrimitive().getAsInt();
                    int step = jsonObject.get("step").getAsJsonPrimitive().getAsInt();

                    return new IntOption(Text.of(name), Text.of(structure.getDesc()), structure.isNeedsRestart(), () -> value, i -> {
                    }, min, max, step, IntOption.STANDARD_FORMATTER);
                }
                case "enum" -> {
                    int selected = jsonObject.get("selected").getAsJsonPrimitive().getAsInt();
                    return new TextEnumOption(Text.of(name), Text.of(structure.getDesc()), structure.isNeedsRestart(), () -> selected, i -> {
                    }, structure.getVariants());
                }
                default -> throw new JsonParseException("Unexpected value: " + typeString);
            }
        }

        @Override
        public List<Option<?>> deserialize(JsonElement json, Type typeOfT, JsonDeserializationContext context) throws JsonParseException {
            if (json instanceof JsonObject jsonObject) {
                var options = new ArrayList<Option<?>>();
                for (var entry : jsonObject.entrySet()) {
                    try {
                        options.add(deserializeOption(entry.getValue().getAsJsonObject(), entry.getKey()));
                    } catch (IllegalStateException e) {
                        throw new JsonParseException(e);
                    }
                }
                return options;
            } else {
                throw new JsonParseException("Tried to deserialize to List<Option<?>>, found a json element that's not an option");
            }
        }

        @Override
        public JsonElement serialize(List<Option<?>> src, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject root = new JsonObject();

            for (Option<?> option : src) {
                root.add(option.name.getString(), serializeOption(option));
            }

            return root;
        }

        private JsonObject serializeOption(Option<?> option) {
            JsonObject root = new JsonObject();
            if (option instanceof BoolOption boolOption) {
                root.addProperty("type", "bool");
                root.addProperty("value", boolOption.get());
            } else if (option instanceof IntOption intOption) {
                root.addProperty("type", "int");
                root.addProperty("value", intOption.get());
                root.addProperty("min", intOption.min);
                root.addProperty("max", intOption.max);
                root.addProperty("step", intOption.step);
            } else if (option instanceof TextEnumOption textEnumOption) {
                root.addProperty("selected", textEnumOption.get());
            } else if (option instanceof FloatOption floatOption) {
                root.addProperty("type", "float");
                root.addProperty("value", floatOption.get());
                root.addProperty("min", floatOption.min);
                root.addProperty("max", floatOption.max);
                root.addProperty("step", floatOption.step);
            } else if (option instanceof EnumOption<?>) {
                throw new IllegalStateException("There should be no EnumOption here!");
            }
            return root;
        }
    }
}
