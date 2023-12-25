package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.util.InputUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(InputUtil.class)
public class InputUtilMixin {
    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public static boolean isKeyPressed(long handle, int code) {
        return Wgpu.keyStates.getOrDefault(code, 1) == 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public static void setCursorParameters(long handler, int inputModeValue, double x, double y) {
        WgpuNative.setCursorPosition(x, y);
        WgpuNative.setCursorMode(inputModeValue);
    }
}
