package cloud.birb.wgpu.mixin;

import cloud.birb.wgpu.rust.Wgpu;
import net.minecraft.client.Keyboard;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.RunArgs;
import net.minecraft.client.util.Window;
import net.minecraft.client.util.WindowProvider;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.InputStream;

@Mixin(MinecraftClient.class)
public class MinecraftClientMixin {

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;setIcon(Ljava/io/InputStream;Ljava/io/InputStream;)V"))
    public void cancelSetIcon(Window window, InputStream icon16, InputStream icon32) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;setFramerateLimit(I)V"))
    public void cancelSetFramerateLimit(Window window, int framerateLimit) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/Mouse;setup(J)V"))
    public void cancelMouseSetup(Mouse mouse, long l) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;getHandle()J"))
    public long cancelGetHandle(Window window) {
        return 0;
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/Keyboard;setup(J)V"))
    public void cancelKeyboardSetup(Keyboard keyboard, long l) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/systems/RenderSystem;initRenderer(IZ)V"))
    public void cancelInitRenderer(int debugVerbosity, boolean debugSync) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;getFramebufferWidth()I"))
    public int redirectWindowFramebufferWidth(Window window) {
        return 0;
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;getFramebufferHeight()I"))
    public int redirectWindowFramebufferHeight(Window window) {
        return 0;
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;setPhase(Ljava/lang/String;)V"))
    public void redirectWindowSetPhase(Window window, String phase) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;setVsync(Z)V"))
    public void setVsync(Window window, boolean vsync) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;setRawMouseMotion(Z)V"))
    public void setRawMouseMotion(Window window, boolean rawMouseMotion) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;logOnGlError()V"))
    public void logOnGlError(Window window) {

    }

    @Inject(method = "onResolutionChanged", at = @At("HEAD"), cancellable = true)
    public void onResolutionChanged(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "openScreen", at = @At("HEAD"), cancellable = true)
    public void openScreen(CallbackInfo ci) { //TODO: Temporary!
        ci.cancel();
    }

    @Inject(method = "<init>", at = @At("TAIL"))
    public void injectWindowHook(RunArgs args, CallbackInfo ci) {
        Wgpu.initializeWindow();
    }

    /**
     * @author Birb
     * 
     * @reason Replace the rendering code
     */
    @Overwrite
    private void render(boolean tick) {

    }

}
