package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.texture.NativeImage;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.io.InputStream;

@Mixin(NativeImage.class)
public class NativeImageMixin {

    @Inject(method = "upload(IIIIIIIZZZZ)V", at = @At("HEAD"), cancellable = true)
    private void upload(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "read(Lnet/minecraft/client/texture/NativeImage$Format;Ljava/io/InputStream;)Lnet/minecraft/client/texture/NativeImage;", at = @At("HEAD"), cancellable = true)
    private static void read(NativeImage.Format format, InputStream inputStream, CallbackInfoReturnable<NativeImage> cir) {
        cir.cancel();
    }

}
