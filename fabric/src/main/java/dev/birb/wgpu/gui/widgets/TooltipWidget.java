package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.WidgetRenderer;
import dev.birb.wgpu.gui.options.Option;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.Element;
import net.minecraft.util.math.ColorHelper;
import net.minecraft.util.math.MathHelper;

public class TooltipWidget extends Widget {
    private Option<?> option;
    private double animation, timer;

    public TooltipWidget(int x, int y) {
        super(x, y, OPTION_WIDTH, 0);
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        Option<?> opt = getHoveredOption(mouseX, mouseY);
        if (opt != null && opt.tooltip == null) opt = null;

        if (option == opt) timer += delta;
        else {
            if (opt != null) animation = 0;
            timer = 0;
        }

        if (opt != null) option = opt;
        else timer = 0;

        if (timer >= 1 || (animation > 0 && option != null)) {
            animation = MathHelper.clamp(animation + delta * 6 * (opt != null ? 1 : -1), 0, 1);

            if (animation > 0) {
                renderer.pushAlpha(animation);
                render(renderer, option);
                renderer.popAlpha();
            }
        }
    }

    private void render(WidgetRenderer renderer, Option<?> option) {
        int tooltipHeight = renderer.wrappedTextHeight(option.tooltip, width - 10) + 10;
        height = tooltipHeight;

        if (option.requiresRestart) height += renderer.textHeight() + 4;

        // Background
        renderer.rect(x + 1, y + 1, x + width - 2, y + height - 2, ColorHelper.Argb.getArgb(225, 0, 0, 0));

        // Outline
        renderer.rect(x, y, x + width, y + 1, ACCENT);
        renderer.rect(x, y + height - 1, x + width, y + height, ACCENT);
        renderer.rect(x, y + 1, x + 1, y + height - 1, ACCENT);
        renderer.rect(x + width - 1, y + 1, x + width, y + height - 1, ACCENT);

        // Text
        renderer.wrappedText(option.tooltip, x + 5, y + 5, WHITE, width - 8);

        // Requires restart
        if (option.requiresRestart) renderer.text("* Requires restart", x + 5, y + tooltipHeight, RED);
    }

    private Option<?> getHoveredOption(int mouseX, int mouseY) {
        Element element = MinecraftClient.getInstance().currentScreen.hoveredElement(mouseX, mouseY).orElse(null);
        if (element instanceof IOptionWidget widget) return widget.getOption();
        return null;
    }
}
