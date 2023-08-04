package dev.birb.wgpu.mixin.render;

import net.minecraft.client.gl.WindowFramebuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(WindowFramebuffer.class)
public class WindowFramebufferMixin {

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    private void initSize(int width, int height) {
    }

}
