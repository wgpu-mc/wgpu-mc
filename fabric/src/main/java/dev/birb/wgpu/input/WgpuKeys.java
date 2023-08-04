package dev.birb.wgpu.input;

import lombok.extern.slf4j.Slf4j;
import org.lwjgl.glfw.GLFW;

@Slf4j
public class WgpuKeys {
    // Keys
    public static final int WGPU_LSHIFT = 118;
    public static final int WGPU_RSHIFT = 139;
    public static final int WGPU_LCONTROL = 117;
    public static final int WGPU_RCONTROL = 138;
    public static final int WGPU_F3 = 39;
    public static final int WGPU_BACKSPACE = 74;
    public static final int WGPU_TAB = 146;
    public static final int WGPU_ESCAPE = 36;
    public static final int WGPU_LEFT = 70;
    public static final int WGPU_UP = 71;
    public static final int WGPU_RIGHT = 72;
    public static final int WGPU_DOWN = 73;
    public static final int WGPU_HOME = 65;
    public static final int WGPU_DELETE = 66;
    public static final int WGPU_END = 67;
    public static final int WGPU_ENTER = 75;
    private static final int WGPU_SPACE = 76;

    // Modifiers
    public static final int WGPU_SHIFT = 0b100;
    public static final int WGPU_CONTROL = 0b100 << 3;
    public static final int WGPU_ALT = 0b100 << 6;
    public static final int WGPU_SUPER = 0b100 << 9;

    // https://www.glfw.org/docs/3.3/group__keys.html

    public static int convertKeyCode(int code) {
        // Numbers
        if (code >= 0 && code <= 9) {
            return code + 48;
        }

        if (code >= 10 && code <= 35) {
            // winit lowercase `a` is 10
            // GLFW  uppercase A is 65, 55 difference
            // This method is used in Wgpu.keyState(..)
            return code + 55;
        }

        return switch (code) {
            case WGPU_LSHIFT -> GLFW.GLFW_KEY_LEFT_SHIFT;
            case WGPU_RSHIFT -> GLFW.GLFW_KEY_RIGHT_SHIFT;
            case WGPU_LCONTROL -> GLFW.GLFW_KEY_LEFT_CONTROL;
            case WGPU_RCONTROL -> GLFW.GLFW_KEY_RIGHT_CONTROL;
            case WGPU_F3 -> GLFW.GLFW_KEY_F3;
            case WGPU_BACKSPACE -> GLFW.GLFW_KEY_BACKSPACE;
            case WGPU_TAB -> GLFW.GLFW_KEY_TAB;
            case WGPU_ESCAPE -> GLFW.GLFW_KEY_ESCAPE;
            case WGPU_LEFT -> GLFW.GLFW_KEY_LEFT;
            case WGPU_UP -> GLFW.GLFW_KEY_UP;
            case WGPU_RIGHT -> GLFW.GLFW_KEY_RIGHT;
            case WGPU_DOWN -> GLFW.GLFW_KEY_DOWN;
            case WGPU_HOME -> GLFW.GLFW_KEY_HOME;
            case WGPU_END -> GLFW.GLFW_KEY_END;
            case WGPU_DELETE -> GLFW.GLFW_KEY_DELETE;
            case WGPU_ENTER -> GLFW.GLFW_KEY_ENTER;
            case WGPU_SPACE -> GLFW.GLFW_KEY_SPACE;
            default -> {
                log.warn("Couldn't convert keycode {}", code);
                yield GLFW.GLFW_KEY_UNKNOWN;
            }
        };
    }


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
