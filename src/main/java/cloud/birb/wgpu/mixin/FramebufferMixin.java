package cloud.birb.wgpu.mixin;

import net.minecraft.client.gl.Framebuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Framebuffer.class)
public class FramebufferMixin {

    @Inject(method = "resize", at = @At("HEAD"), cancellable = true)
    public void resize(int width, int height, boolean getError, CallbackInfo ci) {
        ci.cancel();
    }


}
