package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GameRenderer.class)
public class GameRendererMixin {

    @Inject(method = "reset", at = @At("HEAD"), cancellable = true)
    public void reset(CallbackInfo ci) {
        ci.cancel();
    }

}
