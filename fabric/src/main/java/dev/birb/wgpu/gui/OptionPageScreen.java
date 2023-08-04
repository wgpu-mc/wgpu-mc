package dev.birb.wgpu.gui;

import dev.birb.wgpu.gui.options.Option;
import dev.birb.wgpu.gui.widgets.*;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.Element;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.LiteralText;
import net.minecraft.util.math.MathHelper;

import java.util.ArrayList;
import java.util.List;

public class OptionPageScreen extends Screen {
    private static final WidgetRenderer RENDERER = new WidgetRenderer();
    private static final int MAX_WIDTH = 1000;

    private final Screen parent;

    private final OptionPages pages;
    private OptionPages.Page currentPage;
    private double animation;

    private final List<Widget> widgets = new ArrayList<>();
    private final List<Widget> optionWidgets = new ArrayList<>();
    private final List<Widget> previousOptionWidgets = new ArrayList<>();

    private TooltipWidget tooltipWidget;

    private int previousWidth;
    private int previousHeight;

    public OptionPageScreen(Screen parent) {
        super(new LiteralText("Options"));

        this.parent = parent;
        this.pages = new OptionPages();
        this.currentPage = pages.getDefault();
        this.animation = 1;
    }

    public void setCurrentPage(OptionPages.Page currentPage) {
        if (this.currentPage == currentPage) return;

        this.currentPage = currentPage;

        previousOptionWidgets.clear();
        previousOptionWidgets.addAll(optionWidgets);

        init();

        animation = 0;
    }

    @Override
    protected void init() {
        clearChildren();
        optionWidgets.clear();

        if (width != previousWidth || height != previousHeight) {
            widgets.clear();
            initOtherThanOptions();
        }
        else {
            for (Widget widget : widgets) addSelectableChild(widget);
        }

        // Options
        int x = 8 + TabWidget.WIDTH + 8;
        int y = 8 + TextWidget.HEIGHT + 8;

        int width = getOptimalWidth();

        for (List<Option<?>> group : currentPage) {
            for (Option<?> option : group) {
                add(option.createWidget(alignX(x), y, width - x - 8));
                y += Widget.DEFAULT_HEIGHT;
            }

            y += 4;
        }

        previousWidth = this.width;
        previousHeight = this.height;
    }

    private void initOtherThanOptions() {
        int width = getOptimalWidth();

        // Title
        int x = 8;
        int y = 8;

        y += add(new TextWidget(alignX(x), y, width - 16, new LiteralText("Video Options"))).height + 8;

        // Tabs
        for (OptionPages.Page page : pages) {
            y += add(new TabWidget(alignX(x), y, page, () -> page == this.currentPage)).height;
        }

        // Tooltip
        tooltipWidget = add(new TooltipWidget(0, 0));

        // Buttons in bottom right
        x = width - 8;
        y = height - 8 - Widget.DEFAULT_HEIGHT;
        int w = 100;

        add(new CustomButtonWidget(alignX(x - w), y, () -> new LiteralText(pages.isChanged() ? "Apply and close" : "Close"), w, () -> true, () -> {
            pages.apply();
            onClose();
        }));
        add(new CustomButtonWidget(alignX(x - w - 4 - w), y, () -> new LiteralText("Undo"), w, pages::isChanged, pages::undo));
    }

    private int getOptimalWidth() {
        return Math.min(this.width, (int) (MAX_WIDTH / MinecraftClient.getInstance().getWindow().getScaleFactor()));
    }

    private int alignX(int x) {
        return x + (width - getOptimalWidth()) / 2;
    }

    private Widget getHoveredOptionWidget(int mouseX, int mouseY) {
        Element element = hoveredElement(mouseX, mouseY).orElse(null);
        if (element instanceof IOptionWidget widget) return (Widget) widget;
        return null;
    }

    private <T extends Widget> T add(T widget) {
        addSelectableChild(widget);

        if (widget instanceof IOptionWidget) optionWidgets.add(widget);
        else widgets.add(widget);

        return widget;
    }

    @Override
    public void render(MatrixStack matrices, int mouseX, int mouseY, float delta) {
        renderBackground(matrices);

        var optionWidget = getHoveredOptionWidget(mouseX, mouseY);
        if (optionWidget != null) {
            tooltipWidget.x = optionWidget.x;
            tooltipWidget.y = optionWidget.y + optionWidget.height;
            tooltipWidget.width = optionWidget.width;
        }

        delta /= 20;
        animation = MathHelper.clamp(animation + delta * 6, 0, 1);

        if (animation < 1) {
            RENDERER.pushAlpha(1 - animation);
            for (Widget widget : previousOptionWidgets) widget.render(RENDERER, mouseX, mouseY, delta);
            RENDERER.popAlpha();
        }

        RENDERER.pushAlpha(animation);
        for (Widget widget : optionWidgets) widget.render(RENDERER, mouseX, mouseY, delta);
        RENDERER.popAlpha();

        for (Widget widget : widgets) widget.render(RENDERER, mouseX, mouseY, delta);
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        setDragging(false);
        for (Element element : children()) {
            if (element.mouseReleased(mouseX, mouseY, button)) return true;
        }

        return false;
    }

    @Override
    public void mouseMoved(double mouseX, double mouseY) {
        for (Element element : children()) element.mouseMoved(mouseX, mouseY);
    }

    @Override
    public void onClose() {
        MinecraftClient.getInstance().setScreen(parent);
    }
}
