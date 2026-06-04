package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.ComponentEnumWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.network.chat.Component;

import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

public class ComponentEnumOption extends Option<Integer> {

	public static final Function<ComponentEnumOption, Component> FORMATTER = option -> Component.literal(option.values[option.get()]);
	private final String[] values;

	ComponentEnumOption(Component name, Component tooltip, boolean requiresRestart, Supplier<Integer> getter, Consumer<Integer> setter, String[] values) {
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
		return new ComponentEnumWidget(x, y, width, this);
	}
}
