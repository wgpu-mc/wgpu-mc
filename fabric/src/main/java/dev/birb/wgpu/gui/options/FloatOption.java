package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.FloatWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.network.chat.Component;

import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class FloatOption extends Option<Double> {
	public static final Function<Double, Component> STANDARD_FORMATTER = fl -> Component.literal(String.valueOf(fl));
	public final double min;
	public final double max;
	public final double step;
	public final Function<Double, Component> formatter;

	public FloatOption(Component name, Component tooltip, boolean requiresRestart, Supplier<Double> getter, Consumer<Double> setter, double min, double max, double step, Function<Double, Component> formatter) {
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
		private Function<Double, Component> formatter = STANDARD_FORMATTER;
		private double min;
		private double max;
		private double step = 1;

		public FloatOption.Builder setFormatter(Function<Double, Component> formatter) {
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
