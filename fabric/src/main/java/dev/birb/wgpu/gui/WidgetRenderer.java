package dev.birb.wgpu.gui;

public class WidgetRenderer {
    private final Text context;
    private final FloatStack alphaStack = new FloatArrayList();

    public WidgetRenderer(DrawContext context) {
        this.context = context;
        alphaStack.push(1);
    }

    public void pushAlpha(double alpha) {
        alphaStack.push(alphaStack.peekFloat(0) * (float) alpha);
    }

    public void popAlpha() {
        alphaStack.popFloat();
    }

    public void rect(int x1, int y1, int x2, int y2, int color) {
        context.fill(x1, y1, x2, y2, applyAlpha(color));
    }

    public void text(String text, int x, int y, int color) {
        drawText(text, x, y, applyAlpha(color));
    }
    public void text(Component text, int x, int y, int color) {
        drawText(text.getVisualOrderText(), x, y, applyAlpha(color));
    }
    public void text(FormattedCharSequence text, int x, int y, int color) {
        drawText(text, x, y, applyAlpha(color));
    }

    public void wrappedText(Component text, int x, int y, int color, int maxWidth) {
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
        return ColorHelper.getArgb(
                (int) (ColorHelper.getAlpha(color) * alphaStack.peekFloat(0)),
                ColorHelper.getRed(color),
                ColorHelper.getGreen(color),
                ColorHelper.getBlue(color)
        );
    }

    private void drawText(String text, int x, int y, int color) {
        context.drawText(textRenderer(), text, x, y, color, false);
    }

    private void drawText(FormattedCharSequence text, int x, int y, int color) {
        context.drawText(textRenderer(), text, x, y, color, false);
    }

    private TextRenderer textRenderer() {
        return MinecraftClient.getInstance().textRenderer;
    }
}
