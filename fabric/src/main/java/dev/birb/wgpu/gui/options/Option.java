package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

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
}
