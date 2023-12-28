package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.RunArgs;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientCoreMixin {

    @Shadow
    protected abstract String getWindowTitle();

    @Inject(method = "updateWindowTitle", at = @At("HEAD"), cancellable = true)
    public void modifyUpdateWindowTitle(CallbackInfo ci) {
        WgpuNative.updateWindowTitle(this.getWindowTitle());
        ci.cancel();
    }

    @Inject(method = "getWindowTitle", at = @At(value = "RETURN"), cancellable = true)
    public void getWindowTitleAddWgpu(CallbackInfoReturnable<String> cir) {
        String title = cir.getReturnValue();
        if (!Wgpu.isInitialized()) {
            title += " + Wgpu";
        } else {
            if (Wgpu.getWmIdentity() == null) {
                Wgpu.setWmIdentity(WgpuNative.getBackend());
            }
            title += " + " + Wgpu.getWmIdentity();
        }
        cir.setReturnValue(title);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public boolean shouldRenderAsync() {
        return true;
    }

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
