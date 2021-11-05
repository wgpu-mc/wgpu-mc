package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.mixin.accessors.ThreadExecutorAccessor;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.resource.ResourceManager;
import net.minecraft.resource.ResourceReloader;
import net.minecraft.util.crash.CrashException;
import net.minecraft.util.crash.CrashReport;
import net.minecraft.util.crash.CrashReportSection;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.Queue;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientRenderMixin {

    @Shadow public abstract void startIntegratedServer(String worldName);

    @Final
    @Shadow
    @Nullable
    private Queue<Runnable> renderTaskQueue;
    @Final @Shadow @Nullable private RenderTickCounter renderTickCounter;
    @Shadow private boolean paused;

    @Shadow public abstract ResourceManager getResourceManager();

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
