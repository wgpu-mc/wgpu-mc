package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.RunArgs;
import net.minecraft.util.crash.CrashException;
import net.minecraft.util.crash.CrashReport;
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

    @Inject(method = "run", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/MinecraftClient;render(Z)V"))
    public void run(CallbackInfo ci) {
        if (Wgpu.getException() != null) {
            CrashReport report = new CrashReport(Wgpu.getException().getMessage(), Wgpu.getException());
            report.addElement(
                    "This crash was caused by the Fabric mod Electrum within native Rust code. Please report this crash to the wgpu-mc developers by opening an issue and attaching this crash log at https://github.com/wgpu-mc/wgpu-mc/issues");
            throw new CrashException(report);
        }
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
