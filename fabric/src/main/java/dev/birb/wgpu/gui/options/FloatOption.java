package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.FloatWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.LiteralText;
import net.minecraft.text.Text;

import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class FloatOption extends Option<Double> {
	public static Function<Double, Text> STANDARD_FORMATTER = fl -> new LiteralText(String.valueOf(fl));
	public final double min, max;
	public final double step;
	public Function<Double, Text> formatter;

	public FloatOption(Text name, Text tooltip, boolean requiresRestart, Supplier<Double> getter, Consumer<Double> setter, double min, double max, double step, Function<Double, Text> formatter) {
		super(name, tooltip, requiresRestart, getter, setter);

		this.formatter = formatter;
		this.min = min;
		this.max = max;
		this.step = step;
	}

	@Override
	public Widget createWidget(int x, int y, int width) {
		return new FloatWidget(x, y, width, this);
	}

	public static class Builder extends Option.Builder<FloatOption.Builder, Double> {
		private Function<Double, Text> formatter = STANDARD_FORMATTER;
		private double min, max;
		private double step = 1;

		public FloatOption.Builder setFormatter(Function<Double, Text> formatter) {
			this.formatter = formatter;
			return this;
		}

		public FloatOption.Builder setRange(double min, double max) {
			this.min = min;
			this.max = max;
			return this;
		}

		public FloatOption.Builder setStep(double step) {
			this.step = step;
			return this;
		}

		@Override
		public Option<Double> build() {
			return new FloatOption(name, tooltip, requiresRestart, getter, setter, min, max, step, formatter);
		}
	}
}
