package dev.birb.wgpu.mixin.core;

import net.minecraft.client.Keyboard;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(Keyboard.class)
public class KeyboardMixin {

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setup(long l) {

    }

}
