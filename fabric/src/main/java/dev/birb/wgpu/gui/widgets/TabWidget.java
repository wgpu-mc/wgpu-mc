package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.OptionPageScreen;
import dev.birb.wgpu.gui.OptionPages;
import dev.birb.wgpu.gui.WidgetRenderer;
import net.minecraft.client.MinecraftClient;
import net.minecraft.util.math.MathHelper;

import java.util.function.BooleanSupplier;

public class TabWidget extends Widget {
    public static final int WIDTH = 120;

    private final OptionPages.Page page;
    private final BooleanSupplier selected;

    private double animation;

    public TabWidget(int x, int y, OptionPages.Page page, BooleanSupplier selected) {
        super(x, y, WIDTH, DEFAULT_HEIGHT + 4);

        this.page = page;
        this.selected = selected;
        this.animation = selected.getAsBoolean() ? 1 : 0;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (isMouseOver(mouseX, mouseY) && MinecraftClient.getInstance().currentScreen instanceof OptionPageScreen screen) {
            screen.setCurrentPage(page);
            playClickSound();
            return true;
        }

        return false;
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        animation = MathHelper.clamp(animation + delta * 6 * (selected.getAsBoolean() ? 1 : -1), 0, 1);

        // Background
        renderer.rect(x, y, x + width, y + height, isMouseOver(mouseX, mouseY) ? BG_HOVERED : BG);

        // Text
        renderer.text(page.name, x + 8, centerTextY(renderer), WHITE);

        // Selected
        if (animation > 0) {
            renderer.pushAlpha(animation);
            renderer.rect(x, y, x + 1, y + height, ACCENT);
            renderer.popAlpha();
        }
    }
}
