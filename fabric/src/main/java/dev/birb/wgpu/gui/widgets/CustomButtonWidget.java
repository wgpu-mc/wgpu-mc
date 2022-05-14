package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.WidgetRenderer;
import net.minecraft.text.Text;
import net.minecraft.util.math.MathHelper;

import java.util.function.BooleanSupplier;
import java.util.function.Supplier;

public class CustomButtonWidget extends Widget {
    private final Supplier<Text> textSupplier;
    private final BooleanSupplier visible;
    private final Runnable action;

    private Text text, previousText;
    private double visibleAnimation, textAnimation;

    public CustomButtonWidget(int x, int y, Supplier<Text> textSupplier, int width, BooleanSupplier visible, Runnable action) {
        super(x, y, width, HEIGHT);

        this.textSupplier = textSupplier;
        this.visible = visible;
        this.action = action;
        this.text = textSupplier.get();
        this.visibleAnimation = visible.getAsBoolean() ? 1 : 0;
        this.textAnimation = 1;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (isMouseOver(mouseX, mouseY)) {
            action.run();
            playClickSound();
            return true;
        }

        return false;
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        visibleAnimation = MathHelper.clamp(visibleAnimation + delta * 6 * (visible.getAsBoolean() ? 1 : -1), 0, 1);

        if (visibleAnimation > 0) {
            renderer.pushAlpha(visibleAnimation);

            Text t = textSupplier.get();
            if (!text.equals(t)) {
                previousText = text;
                text = t;
                textAnimation = 0;
            }
            textAnimation = MathHelper.clamp(textAnimation + delta * 6, 0, 1);

            // Background
            renderer.rect(x, y, x + width, y + height, isMouseOver(mouseX, mouseY) ? BG_HOVERED : BG);

            // Text
            if (textAnimation < 1) {
                renderer.pushAlpha(1 - textAnimation);
                renderer.text(previousText, centerX(renderer.textWidth(previousText)), centerTextY(renderer), WHITE);
                renderer.popAlpha();
            }

            renderer.pushAlpha(textAnimation);
            renderer.text(text, centerX(renderer.textWidth(text)), centerTextY(renderer), WHITE);
            renderer.popAlpha();

            renderer.popAlpha();
        }
    }
}
