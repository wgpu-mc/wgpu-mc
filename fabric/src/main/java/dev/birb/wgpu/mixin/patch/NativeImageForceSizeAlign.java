package dev.birb.wgpu.mixin.patch;

import com.mojang.blaze3d.platform.NativeImage;
import net.minecraft.util.Mth;
import org.lwjgl.stb.STBImage;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

@Mixin(NativeImage.class)
public class NativeImageForceSizeAlign {

    @Shadow
    @Final
    private boolean useStbFree;

    @Mutable
    @Shadow
    @Final
    private int width;

    @Mutable
    @Shadow
    @Final
    private int height;

    @ModifyVariable(method = "<init>(Lcom/mojang/blaze3d/platform/NativeImage$Format;IIZ)V", at = @At("HEAD"), argsOnly = true, name = "width")
    private static int modifyWidth1(int width) {
        return Mth.roundToward(width, 256);
    }

    @ModifyVariable(method = "<init>(Lcom/mojang/blaze3d/platform/NativeImage$Format;IIZ)V", at = @At("HEAD"), argsOnly = true, name = "height")
    private static int modifyHeight1(int height) {
        return Mth.roundToward(height, 256);
    }

    @Inject(method = "<init>(Lcom/mojang/blaze3d/platform/NativeImage$Format;IIZJ)V", at = @At("RETURN"))
    private void voidFixWidthHeight(NativeImage.Format format, int width, int height, boolean useStbFree, long pixels, CallbackInfo ci) {
        this.width = Mth.roundToward(width, 256);
        this.height = Mth.roundToward(width, 256);
    }

    @Inject(method = "<init>(Lcom/mojang/blaze3d/platform/NativeImage$Format;IIZJ)V", at = @At("CTOR_HEAD"))
    private static void modify(NativeImage.Format format, int width, int height, boolean useStbFree, long pixels, CallbackInfo ci) {
        var newWidth = Mth.roundToward(width, 256);
        var newHeight = Mth.roundToward(height, 256);

        long newPixels = MemoryUtil.nmemAlloc((long) newWidth * newHeight * format.components());

        for(int y=0;y<height;y++) {
            var rowOffset = pixels + ((long) width * format.components());
            var destRowOffset = newPixels + ((long) newWidth * format.components());

            MemoryUtil.memCopy(pixels + rowOffset, newPixels + destRowOffset, (long) width * format.components());
        }

        if (useStbFree) {
            STBImage.nstbi_image_free(pixels);
        } else {
            MemoryUtil.nmemFree(pixels);
        }

        return pixels;
    }

}
