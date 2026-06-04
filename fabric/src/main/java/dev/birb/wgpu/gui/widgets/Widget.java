package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.WidgetRenderer;
import lombok.Getter;
import lombok.Setter;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.network.chat.Component;

public abstract class Widget extends AbstractWidget {
    public static final int OPTION_WIDTH = 200;
    public static final int DEFAULT_HEIGHT = 21;

    protected static final int BG = getColor(0, 0, 0, 125);
    protected static final int BG_HOVERED = getColor(0, 0, 0, 175);
    protected static final int WHITE = getColor(255, 255, 255, 255);
    protected static final int ACCENT = getColor(225, 220, 144, 255);
    protected static final int RED = getColor(225, 25, 25, 255);

    public int x;
    public int y;
    public int width;
    public int height;
    @Getter
    @Setter
    private boolean focused;

    protected Widget(int x, int y, int width, int height) {
        super(x, y, width, height, Component.literal(""));
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }

    @Override
    public boolean isMouseOver(double mouseX, double mouseY) {
        return mouseX >= x && mouseX <= x + width && mouseY >= y && mouseY <= y + height;
    }

    public abstract void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta);

    @Override
    public void appendNarrations(NarrationMessageBuilder builder) {}

    protected void playClickSound() {
//        Minecraft.getInstance().getSoundManager().play(PositionedSoundInstance.master(SoundEvents.UI_BUTTON_CLICK, 1.0f));
    }

    protected int centerY(int height) {
        return y + (this.height - height) / 2;
    }

    protected int centerTextY(WidgetRenderer renderer) {
        return centerY(renderer.textHeight()) + 1;
    }

    protected int centerX(int width) {
        return x + (this.width - width) / 2;
    }

    protected int alignRight(int width, int totalWidth) {
        return x + totalWidth - width - 6;
    }
    protected int alignRight(int width) {
        return alignRight(width, this.width);
    }

    protected static int getColor(int r, int g, int b, int a) {
        return 0;
//        return ColorHelper.getArgb(a, r, g, b);
    }
}
