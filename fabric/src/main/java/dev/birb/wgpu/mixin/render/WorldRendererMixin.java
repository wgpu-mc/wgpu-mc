package dev.birb.wgpu.mixin.render;

import net.minecraft.client.render.WorldRenderer;
import net.minecraft.resource.ResourceManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(WorldRenderer.class)
public class WorldRendererMixin {

    /**
     * @author
     */
    @Overwrite
    public void renderLightSky() {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void renderDarkSky() {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void renderStars() {

    }

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void reload(ResourceManager manager) {

    }

}
