package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.TextureUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.io.InputStream;
import java.nio.ByteBuffer;

@Mixin(TextureUtil.class)
public class TextureUtilMixin {

    /**
     * @author Allocate textures using wgpu
     */
    @Overwrite
    public static void allocate(NativeImage.GLFormat internalFormat, int id, int maxLevel, int width, int height) {

    }

    @Inject(method = "readAllToByteBuffer", at = @At("HEAD"), cancellable = true)
    private static void readAllToByteBuffer(InputStream inputStream, CallbackInfoReturnable<ByteBuffer> cir) {
        cir.cancel();
    }

}
