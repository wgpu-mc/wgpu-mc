package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import net.minecraft.client.util.InputUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;


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
