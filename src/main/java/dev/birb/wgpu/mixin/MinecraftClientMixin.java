package dev.birb.wgpu.mixin;

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

    @Shadow public abstract void startIntegratedServer(String worldName);

    @Shadow protected abstract String getWindowTitle();

    @Shadow @Nullable public Screen currentScreen;

    @Final @Shadow @Nullable private Queue<Runnable> renderTaskQueue;
    @Final @Shadow @Nullable private RenderTickCounter renderTickCounter;
    @Shadow private boolean paused;

    @Shadow public abstract ResourceManager getResourceManager();

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

    @Inject(method = "openScreen", at = @At("HEAD"), cancellable = true)
    public void openScreen(Screen screen, CallbackInfo ci) { //TODO: Temporary!
        this.currentScreen = screen;
        ci.cancel();
    }

    @Inject(method = "<init>", at = @At("TAIL"))
    public void injectWindowHook(RunArgs args, CallbackInfo ci) {
        MainGameThread.createNewThread((MinecraftClient) (Object) this);

        //Initializing the window hijacks this thread to run the event loop. All communication is now done through channels.
        Wgpu.doEventLoop();
    }

    @Inject(method = "run", at = @At("HEAD"))
    public void injectRun(CallbackInfo ci) {
    }

//
    @Inject(method = "updateWindowTitle", at = @At("HEAD"), cancellable = true)
    public void modifyUpdateWindowTitle(CallbackInfo ci) {
//        Wgpu.updateWindowTitle(this.getWindowTitle());
        ci.cancel();
    }

    @Inject(method = "getWindowTitle", at = @At(value = "RETURN"), cancellable = true)
    public void getWindowTitleAddWgpu(CallbackInfoReturnable<String> cir) {
        cir.setReturnValue(cir.getReturnValue() + " + Wgpu");
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


    /**
     * @author Birb
     *
     * @reason Replace the render loop
     */
    @Overwrite
    private void render(boolean tick) {

        MinecraftClient mc = (MinecraftClient) (Object) this;

//        Runnable runnable;
//        while((runnable = (Runnable)this.renderTaskQueue.poll()) != null) {
//            runnable.run();
//        }

        int k;
        if (tick) {
            Queue<Runnable> tasks = ((ThreadExecutorAccessor) mc).getTasks();

            Runnable item = tasks.peek();
            while(item != null) {
                Runnable task = tasks.remove();

                if(!(task instanceof ResourceReloader)) {
                    task.run();
                }

                item = tasks.peek();
            }

//            this.profiler.pop();
//            this.profiler.push("tick");
//            k = this.renderTickCounter.beginRenderTick(Util.getMeasuringTimeMs());
//            for(int j = 0; j < Math.min(10, k); ++j) {
//                this.profiler.visit("clientTick");
//                this.tick();
                mc.tick();
//            }

//            this.profiler.pop();
        }

    }

    /**
     * @author Birb
     */
    @Overwrite
    public void tick() {
        MinecraftClient mc = (MinecraftClient) (Object) this;

//        if (this.itemUseCooldown > 0) {
//            --this.itemUseCooldown;
//        }

//        this.profiler.push("gui");
//        if (!this.paused) {
//            this.inGameHud.tick();
//        }

//        this.profiler.pop();
//        this.gameRenderer.updateTargetedEntity(1.0F);
//        this.tutorialManager.tick(this.world, this.crosshairTarget);
//        this.profiler.push("gameMode");
        if (!this.paused && mc.world != null) {
            mc.interactionManager.tick();
        }

//        this.profiler.swap("textures");
//        if (this.world != null) {
//            this.textureManager.tick();
//        }

//        if (this.currentScreen == null && this.player != null) {
//            if (this.player.isDead() && !(this.currentScreen instanceof DeathScreen)) {
//                this.openScreen((Screen)null);
//            } else if (this.player.isSleeping() && this.world != null) {
//                this.openScreen(new SleepingChatScreen());
//            }
//        } else if (this.currentScreen != null && this.currentScreen instanceof SleepingChatScreen && !this.player.isSleeping()) {
//            this.openScreen((Screen)null);
//        }

//        if (this.currentScreen != null) {
//            this.attackCooldown = 10000;
//        }
//
//        if (this.currentScreen != null) {
//            Screen.wrapScreenError(() -> {
//                this.currentScreen.tick();
//            }, "Ticking screen", this.currentScreen.getClass().getCanonicalName());
//        }

//        if (!this.options.debugEnabled) {
//            this.inGameHud.resetDebugHudChunk();
//        }
//
//        if (this.overlay == null && (this.currentScreen == null || this.currentScreen.passEvents)) {
//            this.profiler.swap("Keybindings");
//            mc.handleInputEvents();
//            if (this.attackCooldown > 0) {
//                --this.attackCooldown;
//            }
//        }
//
        if (mc.world != null) {
//            this.profiler.swap("gameRenderer");
//            if (!this.paused) {
//                this.gameRenderer.tick();
//            }
//
//            this.profiler.swap("levelRenderer");
//            if (!this.paused) {
//                this.worldRenderer.tick();
//            }
//
//            this.profiler.swap("level");
            if (!this.paused) {
                if (mc.world.getLightningTicksLeft() > 0) {
                    mc.world.setLightningTicksLeft(mc.world.getLightningTicksLeft() - 1);
                }

                mc.world.tickEntities();
            }
//        } else if (this.gameRenderer.getShader() != null) {
//            this.gameRenderer.disableShader();
        }
//
//        if (!this.paused) {
//            this.musicTracker.tick();
//        }
//
//        this.soundManager.tick(this.paused);
        if (mc.world != null) {
//            if (!this.paused) {
//                if (!this.options.joinedFirstServer && this.method_31321()) {
//                    Text text = new TranslatableText("tutorial.socialInteractions.title");
//                    Text text2 = new TranslatableText("tutorial.socialInteractions.description", new Object[]{TutorialManager.getKeybindName("socialInteractions")});
//                    this.field_26843 = new TutorialToast(TutorialToast.Type.SOCIAL_INTERACTIONS, text, text2, true);
//                    this.tutorialManager.method_31365(this.field_26843, 160);
//                    this.options.joinedFirstServer = true;
//                    this.options.write();
//                }
//
//                this.tutorialManager.tick();
//
                try {
                    mc.world.tick(() -> {
                        return true;
                    });
                } catch (Throwable var4) {
                    CrashReport crashReport = CrashReport.create(var4, "Exception in world tick");
                    if (mc.world == null) {
                        CrashReportSection crashReportSection = crashReport.addElement("Affected level");
                        crashReportSection.add("Problem", (Object)"Level is null!");
                    } else {
                        mc.world.addDetailsToCrashReport(crashReport);
                    }

                    throw new CrashException(crashReport);
                }
//            }
//
//            this.profiler.swap("animateTick");
//            if (!this.paused && this.world != null) {
//                this.world.doRandomBlockDisplayTicks(MathHelper.floor(this.player.getX()), MathHelper.floor(this.player.getY()), MathHelper.floor(this.player.getZ()));
//            }
//
//            this.profiler.swap("particles");
//            if (!this.paused) {
//                this.particleManager.tick();
//            }
//        } else if (this.connection != null) {
//            this.profiler.swap("pendingConnection");
//            this.connection.tick();
        }

//        this.profiler.swap("keyboard");
//        this.keyboard.pollDebugCrash();
//        this.profiler.pop();
    }

}
