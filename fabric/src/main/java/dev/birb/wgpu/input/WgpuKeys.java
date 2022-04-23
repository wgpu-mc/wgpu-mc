package dev.birb.wgpu.input;

import org.lwjgl.glfw.GLFW;

public class WgpuKeys {
    public static final int WGPU_LSHIFT = 118;
    public static final int WGPU_RSHIFT = 139;
    public static final int WGPU_LCONTROL = 117;
    public static final int WGPU_RCONTROL = 138;
    public static final int WGPU_F3 = 39;
    public static final int WGPU_BACKSPACE = 74;
    public static int convertKeyCode(int code) {
        int converted = -1;

        if (code >= 10 && code <= 35) {
            // winit lowercase alphabet starts at 10
            // GLFW  uppercase alphabet starts at 65 (+55 from 10), lowercase 32 chars later.
            return code + 55 + 32;
        }
        switch (code) {
            case WGPU_LSHIFT -> converted = GLFW.GLFW_KEY_LEFT_SHIFT;
            case WGPU_RSHIFT -> converted = GLFW.GLFW_KEY_RIGHT_SHIFT;
            case WGPU_LCONTROL -> converted = GLFW.GLFW_KEY_LEFT_CONTROL;
            case WGPU_RCONTROL -> converted = GLFW.GLFW_KEY_RIGHT_CONTROL;
            case WGPU_F3 -> converted = GLFW.GLFW_KEY_F3;
            case WGPU_BACKSPACE -> converted = GLFW.GLFW_KEY_BACKSPACE;
        }
        if(converted == -1) {
            System.out.printf("Couldn't convert %s\n", code);
        }
        return converted;
    }
    public static int WGPU_SHIFT = 0b100;
    public static int WGPU_CONTROL =0b100 << 3;
    public static int WGPU_ALT = 0b100 << 6;
    public static int WGPU_LOGO = 0b100 << 9;


    public static int convertModifiers(int mods) {
        // No point doing 8 comparisons
        if(mods == 0) {
            return 0;
        }
        int output = 0;
        if((mods & WGPU_SHIFT) != 0) {
            output |= GLFW.GLFW_MOD_SHIFT;
        }
        if((mods & WGPU_CONTROL) != 0) {
            output |= GLFW.GLFW_MOD_CONTROL;
        }
        if((mods & WGPU_ALT) != 0) {
            output |=  GLFW.GLFW_MOD_ALT;
        }
        if((mods & WGPU_LOGO) != 0) {
            output |=  GLFW.GLFW_MOD_SUPER;
        }

        return output;
    }
}
