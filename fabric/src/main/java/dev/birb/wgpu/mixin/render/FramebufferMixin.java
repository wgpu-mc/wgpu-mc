package dev.birb.wgpu.mixin.render;

import net.minecraft.client.gl.Framebuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(Framebuffer.class)
public class FramebufferMixin {

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void resize(int width, int height, boolean getError) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void draw(int width, int height) {
        //nah
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void delete() {
        
    }

    /**
     * @author wgpu-mc
     * @reason i'd rather you didn't
     */
    @Overwrite
    public void initFbo(int width, int height, boolean getError) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void checkFramebufferStatus() {

    }

}
