package dev.birb.wgpu;

import dev.birb.wgpu.palette.RustBlockStateAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.block.BlockState;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.math.ColorHelper;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.PalettedContainer;
import net.minecraft.world.chunk.WorldChunk;

public class Utils {
    public static int blendColors(int color1, int color2, double amount) {
        int r = (int) (ColorHelper.Argb.getRed(color1) * amount + ColorHelper.Argb.getRed(color2) * (1 - amount));
        int g = (int) (ColorHelper.Argb.getGreen(color1) * amount + ColorHelper.Argb.getGreen(color2) * (1 - amount));
        int b = (int) (ColorHelper.Argb.getBlue(color1) * amount + ColorHelper.Argb.getBlue(color2) * (1 - amount));
        int a = (int) (ColorHelper.Argb.getAlpha(color1) * amount + ColorHelper.Argb.getAlpha(color2) * (1 - amount));
        return ColorHelper.Argb.getArgb(a, r, g, b);
    }

//     public static void chunkDebug(PackedIntegerArray array, int index) {
// //        ChunkSection section = chunk.getSection(y / 16);
// //        PalettedContainer<BlockState> container = section.getBlockStateContainer();
//         WgpuNative.debugPalette(accessor.getStoragePointer(), index);
//     }

}
