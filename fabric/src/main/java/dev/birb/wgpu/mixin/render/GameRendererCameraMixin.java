package dev.birb.wgpu.mixin.render;

import net.minecraft.client.render.GameRenderer;
import net.minecraft.resource.ResourceFactory;
import net.minecraft.resource.ResourceManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(GameRenderer.class)
public abstract class GameRendererCameraMixin {

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void preloadShaders(ResourceFactory factory) {

    }

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void reload(ResourceManager manager) {
    }

}
