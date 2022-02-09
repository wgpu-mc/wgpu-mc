package dev.birb.wgpu.mixin.core;

import net.minecraft.client.Mouse;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(Mouse.class)
public class MouseMixin {

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setup(long l) {

    }

}
