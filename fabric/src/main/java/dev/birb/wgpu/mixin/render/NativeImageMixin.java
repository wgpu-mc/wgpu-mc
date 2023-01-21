package dev.birb.wgpu.mixin.render;

import net.minecraft.client.texture.NativeImage;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(NativeImage.class)
public class NativeImageMixin {

    @Redirect(method = "upload(IIIIIIIZZZZ)V", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/systems/RenderSystem;isOnRenderThreadOrInit()Z"))
    public boolean isOnRenderThreadOrInit() {
        return true;
    }

}
