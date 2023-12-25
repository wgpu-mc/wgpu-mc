package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuResourceProvider;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.util.Window;
import net.minecraft.client.util.WindowProvider;
import net.minecraft.resource.ReloadableResourceManagerImpl;
import net.minecraft.resource.ResourceType;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientRenderMixin {
    @Redirect(method = "<init>", at = @At(value = "NEW", target = "(Lnet/minecraft/client/MinecraftClient;)Lnet/minecraft/client/util/WindowProvider;"))
    private WindowProvider redirectWindowProvider(MinecraftClient client) throws InstantiationException {
        return (WindowProvider) Wgpu.getUnsafe().allocateInstance(WindowProvider.class);
    }

    @Redirect(method = "<init>", at = @At(value = "NEW", target = "(Lnet/minecraft/resource/ResourceType;)Lnet/minecraft/resource/ReloadableResourceManagerImpl;"))
    private ReloadableResourceManagerImpl redirectWindowProvider(ResourceType type) {
        // todo can't this use fabric's resource loader api or something?
        ReloadableResourceManagerImpl manager = new ReloadableResourceManagerImpl(type);
        WgpuResourceProvider.setManager(manager);

        return manager;
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/WindowProvider;createWindow(Lnet/minecraft/client/WindowSettings;Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/client/util/Window;"))
    private Window redirectWindow(WindowProvider windowProvider, WindowSettings settings, String videoMode, String title) throws InstantiationException {
        //Warning, zero-initialized!
        Window window = (Window) Wgpu.getUnsafe().allocateInstance(Window.class);
        assert WgpuNative.WINDOW != 0;
        window.handle = WgpuNative.WINDOW;
        window.width = 1280;
        window.height = 720;

        // fixes the message saying that it recovers from an invalid resolution
        window.setFramebufferWidth(1280);
        window.setFramebufferHeight(720);

        return window;
    }

    @Inject(method = "render", at = @At("RETURN"))
    public void uploadDrawCalls(boolean tick, CallbackInfo ci) {
        RenderSystem.replayQueue();

        WgpuNative.submitCommands();
    }

}
