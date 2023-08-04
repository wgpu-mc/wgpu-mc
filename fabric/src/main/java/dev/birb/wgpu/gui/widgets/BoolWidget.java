package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.Utils;
import dev.birb.wgpu.gui.WidgetRenderer;
import dev.birb.wgpu.gui.options.BoolOption;
import dev.birb.wgpu.gui.options.Option;
import net.minecraft.util.math.MathHelper;

public class BoolWidget extends Widget implements IOptionWidget {
    private final BoolOption option;

    private double animation;

    public BoolWidget(int x, int y, int width, BoolOption option) {
        super(x, y, width, DEFAULT_HEIGHT);

        this.option = option;
        this.animation = option.get() ? 1 : 0;
    }

    @Override
    public Option<?> getOption() {
        return option;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (isMouseOver(mouseX, mouseY)) {
            option.set(!option.get());
            playClickSound();
            return true;
        }

        return false;
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        animation = MathHelper.clamp(animation + delta * 6 * (option.get() ? 1 : -1), 0, 1);

        renderer.rect(x, y, x + width, y + height, isMouseOver(mouseX, mouseY) ? BG_HOVERED : BG);
        renderer.text(option.getName(), x + 6, centerTextY(renderer), WHITE);

        int color = Utils.blendColors(ACCENT, WHITE, animation);
        int s = renderer.textHeight() + 2;
        int x = alignRight(s);
        int y = centerY(s);

        // Frame
        renderer.rect(x, y, x + s, y + 1, color);
        renderer.rect(x, y + s - 1, x + s, y + s, color);
        renderer.rect(x, y + 1, x + 1, y + s - 1, color);
        renderer.rect(x + s - 1, y + 1, x + s, y + s - 1, color);

        // Middle
        if (animation > 0) {
            renderer.pushAlpha(animation);
            renderer.rect(x + 2, y + 2, x + s - 2, y + s - 2, ACCENT);
            renderer.popAlpha();
        }
    }
}
