package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.entity.EntityState;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuResourceProvider;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.util.Window;
import net.minecraft.client.util.WindowProvider;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.resource.ReloadableResourceManagerImpl;
import net.minecraft.resource.ResourceType;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.Map;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientRenderMixin {
    @Shadow @Nullable public ClientWorld world;
    private static boolean INITIALIZED = false;

    @Redirect(method = "<init>", at = @At(value = "NEW", target = "(Lnet/minecraft/client/MinecraftClient;)Lnet/minecraft/client/util/WindowProvider;"))
    private WindowProvider redirectWindowProvider(MinecraftClient client) throws InstantiationException {
        return (WindowProvider) Wgpu.getUnsafe().allocateInstance(WindowProvider.class);
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/WindowProvider;createWindow(Lnet/minecraft/client/WindowSettings;Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/client/util/Window;"))
    private Window redirectWindow(WindowProvider windowProvider, WindowSettings settings, String videoMode, String title) throws InstantiationException {
        
        //Warning, zero-initialized!
        Window window = (Window) Wgpu.getUnsafe().allocateInstance(Window.class);
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

        if(WgpuMcMod.MAY_INJECT_PART_IDS) {
            ArrayList<Runnable> list = (ArrayList<Runnable>) Wgpu.injectPartIds.clone();
            Wgpu.injectPartIds = new ArrayList<>();

            list.forEach(Runnable::run);
        }

        if(this.world == null) {
            WgpuNative.clearEntities();
        }

        for(Map.Entry<String, EntityState.EntityRenderState> entry : EntityState.renderStates.entrySet()) {
            String entity = entry.getKey();
            EntityState.EntityRenderState state = entry.getValue();

            long time = WgpuNative.setEntityInstanceBuffer(
                    entity,
                    MemoryUtil.memAddress0(state.buffer),
                    state.buffer.position(),
                    MemoryUtil.memAddress0(state.overlays),
                    state.overlays.position(),
                    state.count,
                    state.textureId
            );

            WgpuMcMod.TIME_SPENT_ENTITIES += time;
            WgpuMcMod.ENTRIES++;

            state.buffer.clear();
            state.overlays.clear();

            state.count = 0;
        }
        WgpuNative.submitCommands();
    }

}
