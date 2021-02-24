package cloud.birb.wgpu.mixin;

import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.TextureUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureUtil.class)
public class TextureUtilMixin {

    @Inject(method = "allocate(Lnet/minecraft/client/texture/NativeImage$GLFormat;IIII)V", at = @At("HEAD"), cancellable = true)
    private static void allocate(NativeImage.GLFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        ci.cancel();
    }

}
