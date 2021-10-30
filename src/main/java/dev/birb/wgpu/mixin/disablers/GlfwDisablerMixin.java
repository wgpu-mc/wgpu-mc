package dev.birb.wgpu.mixin.disablers;

import com.mojang.blaze3d.platform.GLX;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.function.LongSupplier;

@Mixin(GLX.class)
public class GlfwDisablerMixin {
    @Inject(method = "_initGlfw", at = @At("HEAD"), cancellable = true)
    private static void initBackendSystem(CallbackInfoReturnable<LongSupplier> cir) {
        cir.setReturnValue(System::nanoTime);
    }

}
