package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.WidgetRenderer;
import net.minecraft.text.Text;

public class TextWidget extends Widget {
    public static final int HEIGHT = Widget.DEFAULT_HEIGHT;

    private final Text text;

    public TextWidget(int x, int y, int width, Text text) {
        super(x, y, width, HEIGHT);

        this.text = text;
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        // Background
        renderer.rect(x, y, x + width, y + height, BG);

        // Text
        renderer.text(text, centerX(renderer.textWidth(text)), centerTextY(renderer), WHITE);
    }
}
