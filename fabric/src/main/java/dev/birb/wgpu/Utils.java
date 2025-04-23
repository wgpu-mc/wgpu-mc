package dev.birb.wgpu;

import net.minecraft.util.math.ColorHelper;

public class Utils {
    public static int blendColors(int color1, int color2, double amount) {
        int r = (int) (ColorHelper.getRed(color1) * amount + ColorHelper.getRed(color2) * (1 - amount));
        int g = (int) (ColorHelper.getGreen(color1) * amount + ColorHelper.getGreen(color2) * (1 - amount));
        int b = (int) (ColorHelper.getBlue(color1) * amount + ColorHelper.getBlue(color2) * (1 - amount));
        int a = (int) (ColorHelper.getAlpha(color1) * amount + ColorHelper.getAlpha(color2) * (1 - amount));
        return ColorHelper.getArgb(a, r, g, b);
    }

}
