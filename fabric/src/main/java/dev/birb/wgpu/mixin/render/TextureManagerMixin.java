package dev.birb.wgpu.mixin.render;

import net.minecraft.client.texture.TextureManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value = TextureManager.class)
public class TextureManagerMixin {

    @Inject(method = "tick", cancellable = true, at = @At("HEAD"))
    public void dontTick(CallbackInfo ci) {
        ci.cancel();
    }

}
