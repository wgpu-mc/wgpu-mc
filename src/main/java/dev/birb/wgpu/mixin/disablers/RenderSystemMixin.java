package dev.birb.wgpu.mixin.disablers;

import com.mojang.blaze3d.systems.RenderCall;
import com.mojang.blaze3d.systems.RenderSystem;
import org.lwjgl.glfw.GLFWErrorCallbackI;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.function.Supplier;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Inject(method = "assertThread", at = @At("HEAD"), cancellable = true)
    private static void assertThread(Supplier<Boolean> check, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "setupDefaultState", at = @At("HEAD"), cancellable = true)
    private static void setupDefaultState(int x, int y, int width, int height, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "recordRenderCall", at = @At("HEAD"), cancellable = true)
    private static void recordRenderCall(RenderCall renderCall, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "activeTexture", at = @At("HEAD"), cancellable = true)
    private static void activeTexture(int texture, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "matrixMode", at = @At("HEAD"), cancellable = true)
    private static void matrixMode(int texture, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "loadIdentity", at = @At("HEAD"), cancellable = true)
    private static void loadIdentity(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "scalef", at = @At("HEAD"), cancellable = true)
    private static void scalef(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "maxSupportedTextureSize", at = @At("HEAD"), cancellable = true)
    private static void scalef(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(0);
    }

    @Inject(method = "matrixMode", at = @At("HEAD"), cancellable = true)
    private static void matrixMode(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "setErrorCallback", at = @At("HEAD"), cancellable = true)
    private static void setErrorCallback(GLFWErrorCallbackI callback, CallbackInfo ci) {
        ci.cancel();
    }

}
