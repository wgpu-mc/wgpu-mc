package dev.birb.wgpu.gui.options;

import com.google.gson.*;
import dev.birb.wgpu.gui.OptionPages;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.ChatFormatting;
import net.minecraft.client.OptionInstance;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.MutableComponent;

import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Supplier;

public abstract class Option<T> {
    public final Component name;
    public final Component tooltip;
    public final boolean requiresRestart;

    private final Supplier<T> getter;
    private final Consumer<T> setter;

    private T value;

    Option(Component name, Component tooltip, boolean requiresRestart, Supplier<T> getter, Consumer<T> setter) {
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

    public Component getName() {
        if (isChanged()) {
            return name.copy().append(" *").withStyle(ChatFormatting.ITALIC);
        }

        return name;
    }

    @SuppressWarnings("unchecked")
    public abstract static class Builder<B extends Builder<B, T>, T> {
        protected Component name;
        protected Component tooltip;
        protected boolean requiresRestart;
        protected Supplier<T> getter;
        protected Consumer<T> setter;

        public B setName(MutableComponent name) {
            this.name = name;
            return (B) this;
        }

        public B setTooltip(Component tooltip, boolean requiresRestart) {
            this.tooltip = tooltip;
            this.requiresRestart = requiresRestart;
            return (B) this;
        }

        public B setTooltip(Component tooltip) {
            return setTooltip(tooltip, false);
        }

        public B setAccessors(Supplier<T> getter, Consumer<T> setter) {
            this.getter = getter;
            this.setter = setter;
            return (B) this;
        }

        // Simple wrapper around minecraft 1.19's SimpleOption, to be reflected on how to handle for wgpu-mc's config
        public B setOption(OptionInstance<T> option, Consumer<T> callback) {
            this.getter = option::get;
            this.setter = v -> {
                option.set(v);
                callback.accept(v);
            };

            return (B) this;
        }

        public B setOption(OptionInstance<T> option) {
            this.getter = option::get;
            this.setter = option::set;
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
                    return new BoolOption(Component.literal(name), Component.literal(structure.getDesc()), structure.isNeedsRestart(), () -> value, bool -> {
                    });
                }
                case "float" -> {
                    double value = jsonObject.get("value").getAsJsonPrimitive().getAsDouble();
                    double min = jsonObject.get("min").getAsJsonPrimitive().getAsDouble();
                    double max = jsonObject.get("max").getAsJsonPrimitive().getAsDouble();
                    double step = jsonObject.get("step").getAsJsonPrimitive().getAsDouble();

                    return new FloatOption(Component.literal(name), Component.literal(structure.getDesc()), structure.isNeedsRestart(), () -> value, i -> {
                    }, min, max, step, FloatOption.STANDARD_FORMATTER);
                }
                case "int" -> {
                    int value = jsonObject.get("value").getAsJsonPrimitive().getAsInt();
                    int min = jsonObject.get("min").getAsJsonPrimitive().getAsInt();
                    int max = jsonObject.get("max").getAsJsonPrimitive().getAsInt();
                    int step = jsonObject.get("step").getAsJsonPrimitive().getAsInt();

                    return new IntOption(Component.literal(name), Component.literal(structure.getDesc()), structure.isNeedsRestart(), () -> value, i -> {
                    }, min, max, step, IntOption.STANDARD_FORMATTER);
                }
                case "enum" -> {
                    int selected = jsonObject.get("selected").getAsJsonPrimitive().getAsInt();
                    return new ComponentEnumOption(Component.literal(name), Component.literal(structure.getDesc()), structure.isNeedsRestart(), () -> selected, i -> {
                    }, structure.getVariants());
                }
                default -> throw new JsonParseException("Unexpected value: " + typeString);
            }
        }

        @Override
        public List<Option<?>> deserialize(JsonElement jsonElement, Type type, JsonDeserializationContext jsonDeserializationContext) throws JsonParseException {
            if (jsonElement instanceof JsonObject jsonObject) {
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

//        @Override
        public JsonElement serialize(List<Option<?>> src, Type typeOfSrc, JsonSerializationConComponent conComponent) {
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
            } else if (option instanceof ComponentEnumOption ComponentEnumOption) {
                root.addProperty("selected", ComponentEnumOption.get());
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

        @Override
        public JsonElement serialize(List<Option<?>> options, Type type, JsonSerializationContext jsonSerializationContext) {
            return null;
        }
    }
}
