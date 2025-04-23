package dev.birb.wgpu.mixin.render;

import net.minecraft.client.render.WorldRenderer;
import net.minecraft.resource.ResourceManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(WorldRenderer.class)
public abstract class WorldRendererMixin {

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void reload(ResourceManager manager) {
    }


//    @Inject(method = "reload", cancellable = true, at = @At("HEAD"))
//    public void reload(CallbackInfo ci) {
//        WgpuNative.reloadStorage(this.client.options.getClampedViewDistance(),this.world.getBottomSectionCoord());
//    }
}
