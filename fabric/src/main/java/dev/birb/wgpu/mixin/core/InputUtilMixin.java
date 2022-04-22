package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.util.InputUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(InputUtil.class)
public class InputUtilMixin {
    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static boolean isKeyPressed(long handle, int code) {
//        System.out.printf("IsKeyPressed(%s) = %s\n", code, Wgpu.keyStates.get(code));
        return Wgpu.keyStates.getOrDefault(code, 1) == 0;
    }


}
