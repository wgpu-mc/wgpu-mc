package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.TextEnumWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.Text;

import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class TextEnumOption extends Option<Integer> {

	public static final Function<TextEnumOption, Text> FORMATTER = option -> Text.of(option.values[option.get()]);
	private final String[] values;

	TextEnumOption(Text name, Text tooltip, boolean requiresRestart, Supplier<Integer> getter, Consumer<Integer> setter, String[] values) {
		super(name, tooltip, requiresRestart, getter, setter);
		this.values = values;
	}

	public int cycle(int direction) {
		int index = get();
		index += direction;
		while (index < 0) {
			index += values.length;
		}
		index %= values.length;
		set(index);
		return index;
	}

	@Override
	public Widget createWidget(int x, int y, int width) {
		return new TextEnumWidget(x, y, width, this);
	}
}
