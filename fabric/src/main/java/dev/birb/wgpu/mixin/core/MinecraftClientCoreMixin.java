package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.RunArgs;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientCoreMixin {

    @Inject(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/resource/ResourceReloadLogger;reload(Lnet/minecraft/client/resource/ResourceReloadLogger$ReloadReason;Ljava/util/List;)V", shift = At.Shift.AFTER))
    public void injectWindowHook(RunArgs args, CallbackInfo ci) {
        // Register blocks
        Wgpu.setMayInitialize(true);
    }

    @Inject(method = "scheduleStop", at = @At("HEAD"))
    public void scheduleRustStop(CallbackInfo ci) {
        WgpuNative.scheduleStop();
    }
}
