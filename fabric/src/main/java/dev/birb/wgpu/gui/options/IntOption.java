package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.IntWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.network.chat.Component;

import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class IntOption extends Option<Integer> {

    public static final Function<Integer, Component> STANDARD_FORMATTER = integer -> Component.literal(String.valueOf(integer));
	public final Function<Integer, Component> formatter;
    public final int min;
    public final int max;
	public final int step;

	public IntOption(Component name, Component tooltip, boolean requiresRestart, Supplier<Integer> getter, Consumer<Integer> setter, int min, int max, int step, Function<Integer, Component> formatter) {
		super(name, tooltip, requiresRestart, getter, setter);

		this.formatter = formatter;
		this.min = min;
		this.max = max;
		this.step = step;
	}

    @Override
    public Widget createWidget(int x, int y, int width) {
        return new IntWidget(x, y, width, this);
    }

    public static class Builder extends Option.Builder<Builder, Integer> {
		private Function<Integer, Component> formatter = STANDARD_FORMATTER;
        private int min;
        private int max;
        private int step = 1;

        public Builder setFormatter(Function<Integer, Component> formatter) {
            this.formatter = formatter;
            return this;
        }

        public Builder setRange(int min, int max) {
            this.min = min;
            this.max = max;
            return this;
        }

        public Builder setStep(int step) {
            this.step = step;
            return this;
        }

        @Override
        public Option<Integer> build() {
            return new IntOption(name, tooltip, requiresRestart, getter, setter, min, max, step, formatter);
        }
    }
}
