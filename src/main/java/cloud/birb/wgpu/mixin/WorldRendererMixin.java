package cloud.birb.wgpu.mixin;

import net.minecraft.client.render.WorldRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(WorldRenderer.class)
public class WorldRendererMixin {

    @Inject(method = "renderStars()V", at = @At("HEAD"), cancellable = true)
    private void renderStars(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderLightSky()V", at = @At("HEAD"), cancellable = true)
    private void renderLightSky(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderDarkSky()V", at = @At("HEAD"), cancellable = true)
    private void renderDarkSky(CallbackInfo ci) {
        ci.cancel();
    }

}
