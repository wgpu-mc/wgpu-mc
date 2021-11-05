package dev.birb.wgpu.mixin.disablers;

import dev.birb.wgpu.game.MainGameThread;
import dev.birb.wgpu.mixin.accessors.ThreadExecutorAccessor;
import dev.birb.wgpu.rust.Wgpu;
import net.minecraft.client.Keyboard;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.RunArgs;
import net.minecraft.client.gui.screen.DeathScreen;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.SleepingChatScreen;
import net.minecraft.client.render.BufferBuilderStorage;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.client.toast.TutorialToast;
import net.minecraft.client.tutorial.TutorialManager;
import net.minecraft.client.util.Window;
import net.minecraft.client.util.WindowProvider;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.resource.ResourceManager;
import net.minecraft.resource.ResourceReloader;
import net.minecraft.text.Text;
import net.minecraft.text.TranslatableText;
import net.minecraft.util.Util;
import net.minecraft.util.crash.CrashException;
import net.minecraft.util.crash.CrashReport;
import net.minecraft.util.crash.CrashReportSection;
import net.minecraft.util.math.MathHelper;
import net.minecraft.world.World;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import sun.misc.Unsafe;

import java.io.InputStream;
import java.lang.reflect.Field;
import java.util.Queue;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientMixin {

    private static Unsafe UNSAFE;

    static {
        Field f = null; //Internal reference
        try {
            f = Unsafe.class.getDeclaredField("theUnsafe");
            f.setAccessible(true);
            UNSAFE = (Unsafe) f.get(null);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

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

//    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/systems/RenderSystem;initBackendSystem()V"))
//    public LongSupplier cancelInitBackendSystem() {
//        return () -> {
//            return 0L;
//        };
//    }

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

//    @Redirect(method = "<init>", at = @At())

    @Inject(method = "onResolutionChanged", at = @At("HEAD"), cancellable = true)
    public void onResolutionChanged(CallbackInfo ci) {
        ci.cancel();
    }

    @Redirect(method = "<init>", at = @At(value = "NEW", target = "net/minecraft/client/util/WindowProvider"))
    private WindowProvider redirectWindowProvider(MinecraftClient client) throws InstantiationException {
        return (WindowProvider) UNSAFE.allocateInstance(WindowProvider.class);
    }

    @Redirect(method = "<init>", at = @At(value = "NEW", target = "net/minecraft/client/render/GameRenderer"))
    private GameRenderer redirectGameRenderer(MinecraftClient client, ResourceManager manager, BufferBuilderStorage buffers) throws InstantiationException {
        return (GameRenderer) UNSAFE.allocateInstance(GameRenderer.class);
    }

    @Inject(method = "setWorld", cancellable = true, at = @At("HEAD"))
    public void cancelSetWorld(ClientWorld world, CallbackInfo ci) {
        ci.cancel();
    }

}
