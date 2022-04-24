package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.mixin.accessors.ThreadExecutorAccessor;
import dev.birb.wgpu.mixin.accessors.WindowAccessor;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuResourceProvider;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.client.util.Window;
import net.minecraft.client.util.WindowProvider;
import net.minecraft.resource.ReloadableResourceManagerImpl;
import net.minecraft.resource.ResourceManager;
import net.minecraft.resource.ResourceReloader;
import net.minecraft.resource.ResourceType;
import net.minecraft.util.crash.CrashException;
import net.minecraft.util.crash.CrashReport;
import net.minecraft.util.crash.CrashReportSection;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import sun.misc.Unsafe;

import java.lang.reflect.Field;
import java.util.Queue;

import static dev.birb.wgpu.render.Wgpu.UNSAFE;
//import jdk.internal.misc.Unsafe;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientRenderMixin {

    private static boolean INITIALIZED = false;

    @Shadow public abstract void startIntegratedServer(String worldName);

    @Final
    @Shadow
    @Nullable
    private Queue<Runnable> renderTaskQueue;
    @Final @Shadow @Nullable private RenderTickCounter renderTickCounter;
    @Shadow private boolean paused;

    @Shadow public abstract ResourceManager getResourceManager();

    @Redirect(method = "<init>", at = @At(value = "NEW", target = "net/minecraft/client/util/WindowProvider"))
    private WindowProvider redirectWindowProvider(MinecraftClient client) throws InstantiationException {
        return (WindowProvider) UNSAFE.allocateInstance(WindowProvider.class);
    }

    @Redirect(method = "<init>", at = @At(value = "NEW", target = "net/minecraft/resource/ReloadableResourceManagerImpl"))
    private ReloadableResourceManagerImpl redirectWindowProvider(ResourceType type) {
        ReloadableResourceManagerImpl manager = new ReloadableResourceManagerImpl(type);
        WgpuResourceProvider.manager = manager;

        return manager;
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/WindowProvider;createWindow(Lnet/minecraft/client/WindowSettings;Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/client/util/Window;"))
    private Window redirectWindow(WindowProvider windowProvider, WindowSettings settings, String videoMode, String title) throws InstantiationException {
        //Warning, zero-initialized!
        Window window = (Window) UNSAFE.allocateInstance(Window.class);
        WindowAccessor accessor = (WindowAccessor) (Object) window;
        accessor.setWidth(1280);
        accessor.setHeight(720);
        return window;
    }

    @Inject(method = "render", at = @At("RETURN"))
    public void uploadDrawCalls(boolean tick, CallbackInfo ci) {
        RenderSystem.replayQueue();

        if(Wgpu.INITIALIZED) {
            WgpuNative.submitCommands();
            WgpuNative.setWorldRenderState(MinecraftClient.getInstance().world != null);
        }
    }

}
