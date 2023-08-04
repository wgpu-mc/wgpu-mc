package dev.birb.wgpu.gui;

import it.unimi.dsi.fastutil.floats.FloatArrayList;
import it.unimi.dsi.fastutil.floats.FloatStack;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gui.DrawableHelper;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.OrderedText;
import net.minecraft.text.StringVisitable;
import net.minecraft.text.Text;
import net.minecraft.util.math.ColorHelper;
import net.minecraft.util.math.Matrix4f;

public class WidgetRenderer extends DrawableHelper {
    private static final MatrixStack MATRICES = new MatrixStack();
    private static final Matrix4f MATRIX = new Matrix4f();

    static {
        MATRIX.loadIdentity();
    }

    private final FloatStack alphaStack = new FloatArrayList();

    public WidgetRenderer() {
        alphaStack.push(1);
    }

    public void pushAlpha(double alpha) {
        alphaStack.push(alphaStack.peekFloat(0) * (float) alpha);
    }

    public void popAlpha() {
        alphaStack.popFloat();
    }

    public void rect(int x1, int y1, int x2, int y2, int color) {
        fill(MATRICES, x1, y1, x2, y2, applyAlpha(color));
    }

    public void text(String text, int x, int y, int color) {
        drawText(text, x, y, applyAlpha(color));
    }
    public void text(Text text, int x, int y, int color) {
        drawText(text.asOrderedText(), x, y, applyAlpha(color));
    }
    public void text(OrderedText text, int x, int y, int color) {
        drawText(text, x, y, applyAlpha(color));
    }

    public void wrappedText(Text text, int x, int y, int color, int maxWidth) {
        color = applyAlpha(color);

        for (OrderedText orderedText : textRenderer().wrapLines(text, maxWidth)) {
            drawText(orderedText, x, y, color);
            y += textHeight();
        }
    }

    public int wrappedTextHeight(Text text, int maxWidth) {
        return textRenderer().wrapLines(text, maxWidth).size() * textHeight();
    }

    public StringVisitable trimText(StringVisitable text, int width) {
        return textRenderer().trimToWidth(text, width);
    }

    public int textWidth(String text) {
        return textRenderer().getWidth(text);
    }
    public int textWidth(Text text) {
        return textRenderer().getWidth(text);
    }

    public int textHeight() {
        return textRenderer().fontHeight;
    }

    private int applyAlpha(int color) {
        return ColorHelper.Argb.getArgb(
                (int) (ColorHelper.Argb.getAlpha(color) * alphaStack.peekFloat(0)),
                ColorHelper.Argb.getRed(color),
                ColorHelper.Argb.getGreen(color),
                ColorHelper.Argb.getBlue(color)
        );
    }

    private void drawText(String text, float x, float y, int color) {
        VertexConsumerProvider.Immediate immediate = VertexConsumerProvider.immediate(Tessellator.getInstance().getBuffer());
        textRenderer().drawLayer(text, x, y, color, false, MATRIX, immediate, false, 0, LightmapTextureManager.MAX_LIGHT_COORDINATE);
        immediate.draw();
    }
    private void drawText(OrderedText text, float x, float y, int color) {
        VertexConsumerProvider.Immediate immediate = VertexConsumerProvider.immediate(Tessellator.getInstance().getBuffer());
        textRenderer().drawLayer(text, x, y, color, false, MATRIX, immediate, false, 0, LightmapTextureManager.MAX_LIGHT_COORDINATE);
        immediate.draw();
    }

    private TextRenderer textRenderer() {
        return MinecraftClient.getInstance().textRenderer;
    }
}
