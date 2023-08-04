package dev.birb.wgpu;

import net.minecraft.util.math.ColorHelper;

public class Utils {
    public static int blendColors(int color1, int color2, double amount) {
        int r = (int) (ColorHelper.Argb.getRed(color1) * amount + ColorHelper.Argb.getRed(color2) * (1 - amount));
        int g = (int) (ColorHelper.Argb.getGreen(color1) * amount + ColorHelper.Argb.getGreen(color2) * (1 - amount));
        int b = (int) (ColorHelper.Argb.getBlue(color1) * amount + ColorHelper.Argb.getBlue(color2) * (1 - amount));
        int a = (int) (ColorHelper.Argb.getAlpha(color1) * amount + ColorHelper.Argb.getAlpha(color2) * (1 - amount));
        return ColorHelper.Argb.getArgb(a, r, g, b);
    }

}
