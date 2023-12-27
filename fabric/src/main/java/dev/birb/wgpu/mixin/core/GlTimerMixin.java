package dev.birb.wgpu.mixin.core;

import net.minecraft.client.gl.GlTimer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Optional;

@Mixin(GlTimer.class)
public class GlTimerMixin {

    @Inject(method = "getInstance", at = @At("HEAD"), cancellable = true)
    private static void dontGetInstance(CallbackInfoReturnable<Optional<GlTimer>> cir) {
        cir.setReturnValue(Optional.empty());
    }

}
