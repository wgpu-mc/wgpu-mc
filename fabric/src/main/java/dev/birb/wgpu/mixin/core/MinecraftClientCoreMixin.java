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

import static dev.birb.wgpu.render.Wgpu.wmIdentity;

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
        if(!Wgpu.INITIALIZED) {
            title += " + Wgpu";
        } else {
            if (wmIdentity == null) {
                wmIdentity = WgpuNative.getBackend();
            }
            title += " + " + wmIdentity;
        }
        cir.setReturnValue(title);
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public boolean shouldRenderAsync() {
        return true;
    }

    @Inject(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/resource/ResourceReloadLogger;reload(Lnet/minecraft/client/resource/ResourceReloadLogger$ReloadReason;Ljava/util/List;)V", shift = At.Shift.AFTER))
    public void injectWindowHook(RunArgs args, CallbackInfo ci) {
        //Register blocks
        Wgpu.MAY_INITIALIZE = true;
    }

}
