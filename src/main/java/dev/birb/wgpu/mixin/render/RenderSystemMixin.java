package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.rust.Wgpu;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Inject(method = "getApiDescription", at = @At("HEAD"), cancellable = true)
    private static void getApiDescription(CallbackInfoReturnable<String> cir) {
        cir.setReturnValue("wgpu-mc 0.1");
    }

    @Inject(method = "getBackendDescription", at = @At("HEAD"), cancellable = true)
    private static void getBackendDescription(CallbackInfoReturnable<String> cir) {
        cir.setReturnValue(Wgpu.getBackend());
    }

}
