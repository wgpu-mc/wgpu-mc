package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.EnumWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.LiteralText;
import net.minecraft.text.Text;

import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class EnumOption<T extends Enum<T>> extends Option<T> {
    public final Function<T, Text> formatter;

    private final List<T> values;

    private EnumOption(Text name, Class<T> enumClass, Text tooltip, boolean requiresRestart, Supplier<T> getter, Consumer<T> setter, Function<T, Text> formatter) {
        super(name, tooltip, requiresRestart, getter, setter);

        this.formatter = formatter;
        this.values = new ArrayList<>(EnumSet.allOf(enumClass));
    }

    public T cycle(int direction) {
        for (int i = 0; i < values.size(); i++) {
            if (values.get(i) == get()) {
                i += direction;

                if (i >= values.size()) i = 0;
                else if (i < 0) i = values.size() - 1;

                return values.get(i);
            }
        }

        throw new IllegalStateException("This should never happen");
    }

    @Override
    public Widget createWidget(int x, int y, int width) {
        return new EnumWidget<>(x, y, width, this);
    }

    public static class Builder<T extends Enum<T>> extends Option.Builder<Builder<T>, T> {
        private Function<T, Text> formatter = t -> new LiteralText(t.toString());
        private final Class<T> enumClass;

        public Builder(Class<T> enumClass) {
            this.enumClass = enumClass;
        }

        public Builder<T> setFormatter(Function<T, Text> formatter) {
            this.formatter = formatter;
            return this;
        }

        @Override
        public Option<T> build() {
            return new EnumOption<>(name, enumClass, tooltip, requiresRestart, getter, setter, formatter);
        }
    }
}
