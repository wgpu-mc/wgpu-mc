package dev.birb.wgpu.gui.options;

import dev.birb.wgpu.gui.widgets.BoolWidget;
import dev.birb.wgpu.gui.widgets.Widget;
import net.minecraft.text.Text;

import java.util.function.Consumer;
import java.util.function.Supplier;

public class BoolOption extends Option<Boolean> {
    private BoolOption(Text name, Text tooltip, boolean requiresRestart, Supplier<Boolean> getter, Consumer<Boolean> setter) {
        super(name, tooltip, requiresRestart, getter, setter);
    }

    @Override
    public Widget createWidget(int x, int y, int width) {
        return new BoolWidget(x, y, width, this);
    }

    public static class Builder extends Option.Builder<Builder, Boolean> {
        @Override
        public Option<Boolean> build() {
            return new BoolOption(name, tooltip, requiresRestart, getter, setter);
        }
    }
}
