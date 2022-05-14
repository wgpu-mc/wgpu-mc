package dev.birb.wgpu.gui.widgets;

import dev.birb.wgpu.gui.WidgetRenderer;
import dev.birb.wgpu.gui.options.EnumOption;
import dev.birb.wgpu.gui.options.Option;
import net.minecraft.text.Text;
import net.minecraft.util.math.MathHelper;
import org.lwjgl.glfw.GLFW;

public class EnumWidget<T extends Enum<?>> extends Widget implements IOptionWidget {
    private final EnumOption<T> option;

    private Text valueName, previousValueName;
    private double animation;

    public EnumWidget(int x, int y, int width, EnumOption<T> option) {
        super(x, y, width, HEIGHT);

        this.option = option;
        this.valueName = option.formatter.apply(option.get());
        this.animation = 1;
    }

    @Override
    public Option<?> getOption() {
        return option;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (isMouseOver(mouseX, mouseY)) {
            option.set(option.cycle(button == GLFW.GLFW_MOUSE_BUTTON_LEFT ? 1 : -1));

            previousValueName = valueName;
            valueName = option.formatter.apply(option.get());
            animation = 0;

            playClickSound();
            return true;
        }

        return false;
    }

    @Override
    public void render(WidgetRenderer renderer, int mouseX, int mouseY, double delta) {
        animation = MathHelper.clamp(animation + delta * 6, 0, 1);

        // Background
        renderer.rect(x, y, x + width, y + height, isMouseOver(mouseX, mouseY) ? BG_HOVERED : BG);

        // Name
        renderer.text(option.getName(), x + 6, centerTextY(renderer), WHITE);

        // Value
        if (animation < 1) {
            renderer.pushAlpha(1 - animation);
            renderer.text(previousValueName, alignRight(renderer.textWidth(previousValueName)), centerTextY(renderer), WHITE);
            renderer.popAlpha();
        }

        renderer.pushAlpha(animation);
        renderer.text(valueName, alignRight(renderer.textWidth(valueName)), centerTextY(renderer), WHITE);
        renderer.popAlpha();
    }
}
