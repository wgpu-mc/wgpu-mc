package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.texture.NativeImageBackedTexture;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(NativeImageBackedTexture.class)
public class NativeImageBackedTextureMixin {

    @Inject(method = "upload", at = @At("HEAD"), cancellable = true)
    private void upload(CallbackInfo ci) {
        ci.cancel();
    }

}
