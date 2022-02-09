package dev.birb.wgpu.mixin.render;

import net.minecraft.client.render.WorldRenderer;
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

}
