package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import net.minecraft.client.Keyboard;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Keyboard.class)
public class KeyboardMixin {

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setup(long l) {

    }

    @Redirect(method = "onKey", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/InputUtil;isKeyPressed(JI)Z"))
    public boolean redirectIsKeyPressed(long handle, int code) {
        return Wgpu.keyState.get(code) == 1;
    }

}
