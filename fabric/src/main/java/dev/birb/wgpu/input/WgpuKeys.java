package dev.birb.wgpu.input;

import org.lwjgl.glfw.GLFW;

public class WgpuKeys {
    // Modifiers
    public static final int WGPU_SHIFT = 0b100;
    public static final int WGPU_CONTROL = 0b100 << 3;
    public static final int WGPU_ALT = 0b100 << 6;
    public static final int WGPU_SUPER = 0b100 << 9;

    // https://www.glfw.org/docs/3.3/group__keys.html

    // TODO move to rust?
    public static int convertModifiers(int mods) {
        // No point doing 8 comparisons
        if (mods == 0) {
            return 0;
        }
        int output = 0;
        if ((mods & WGPU_SHIFT) != 0) {
            output |= GLFW.GLFW_MOD_SHIFT;
        }
        if ((mods & WGPU_CONTROL) != 0) {
            output |= GLFW.GLFW_MOD_CONTROL;
        }
        if ((mods & WGPU_ALT) != 0) {
            output |= GLFW.GLFW_MOD_ALT;
        }
        if ((mods & WGPU_SUPER) != 0) {
            output |= GLFW.GLFW_MOD_SUPER;
        }

        return output;
    }
}
