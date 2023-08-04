package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.WidgetRenderer;
import dev.birb.wgpu.gui.options.IntOption;
import dev.birb.wgpu.gui.options.Option;
import net.minecraft.client.util.TextCollector;
import net.minecraft.text.LiteralText;
import net.minecraft.text.StringVisitable;
import net.minecraft.text.Text;
import net.minecraft.util.Language;

public class IntWidget extends Widget implements IOptionWidget {
    private final IntOption option;

    private boolean dragging;

    public IntWidget(int x, int y, int width, IntOption option) {
        super(x, y, width, DEFAULT_HEIGHT);

        this.option = option;
    }

    @Override
    public Option<?> getOption() {
        return option;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (isMouseOver(mouseX, mouseY)) {
            dragging = true;
            calculateValue((int) mouseX);
            return false;
        }

        return false;
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        if (dragging) {
            dragging = false;
            return true;
        }

        return false;
    }

    @Override
    public void mouseMoved(double mouseX, double mouseY) {
        if (dragging) calculateValue((int) mouseX);
    }

    private void calculateValue(int mouseX) {
        int w = width / 2;
        mouseX -= x + w;
        w -= 6;

        if (mouseX < 0) option.set(option.min);
        else if (mouseX > w) option.set(option.max);
        else {
            double value = (double) mouseX / w * (option.max - option.min) + option.min;
            option.set((int) Math.round(value / option.step) * option.step);
        }
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        boolean hovered = isMouseOver(mouseX, mouseY) || dragging;

        // Background
        renderer.rect(x, y, x + width, y + height, hovered ? BG_HOVERED : BG);

        int halfWidth = width / 2;

        // Name
        if (hovered && renderer.textWidth(option.getName()) > width / 3) {
            TextCollector collector = new TextCollector();
            collector.add(renderer.trimText(option.getName(), width / 3));
            collector.add(StringVisitable.plain("..."));
            renderer.text(Language.getInstance().reorder(collector.getCombined()), x + 6, centerTextY(renderer), WHITE);
        }
        else renderer.text(option.getName(), x + 6, centerTextY(renderer), WHITE);

        // Value
        Text valueText = hovered ? new LiteralText(String.valueOf(option.get())) : option.formatter.apply(option.get());
        renderer.text(valueText, alignRight(renderer.textWidth(valueText), hovered ? halfWidth : width), centerTextY(renderer), WHITE);

        if (hovered) {
            // Track
            renderer.rect(x + halfWidth, centerY(1), x + width - 6, centerY(1) + 1, WHITE);

            // Handle
            int x = this.x + halfWidth + getHandleX();
            int h = renderer.textHeight() + 2;
            renderer.rect(x, centerY(h), x + 3, centerY(h) + h, WHITE);
        }
    }

    private int getHandleX() {
        double delta = (double) (option.get() - option.min) / (option.max - option.min);
        return (int) (delta * (width / 2 - 6)) - 1;
    }
}
