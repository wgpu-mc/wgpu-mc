package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.IntWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.LiteralText;
import net.minecraft.text.Text;

import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class IntOption extends Option<Integer> {

	public static Function<Integer, Text> STANDARD_FORMATTER = integer -> new LiteralText(String.valueOf(integer));
	public final Function<Integer, Text> formatter;
	public final int min, max;
	public final int step;

	public IntOption(Text name, Text tooltip, boolean requiresRestart, Supplier<Integer> getter, Consumer<Integer> setter, int min, int max, int step, Function<Integer, Text> formatter) {
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
		private Function<Integer, Text> formatter = STANDARD_FORMATTER;
		private int min, max;
        private int step = 1;

        public Builder setFormatter(Function<Integer, Text> formatter) {
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
