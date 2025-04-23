package dev.birb.wgpu.mixin.render;

import net.minecraft.client.render.GameRenderer;
import org.joml.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GameRenderer.class)
public abstract class GameRendererMixin {

    @Inject(at = @At("HEAD"), method = "renderHand", cancellable = true)
    public void renderHand(float tickProgress, boolean sleeping, Matrix4f positionMatrix, CallbackInfo ci) {
        ci.cancel();
    }

//    @Inject(at = @At("RETURN"), method = "render")
//    public void render(RenderTickCounter tickCounter, boolean tick, CallbackInfo ci) {
//        WgpuNative.render(tickCounter.getDynamicDeltaTicks(), 0, tick);
//    }

}
