package dev.birb.wgpu.render;

import dev.birb.wgpu.mixin.core.KeyboardMixin;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.client.MinecraftClient;
import org.lwjgl.glfw.GLFW;

import java.util.HashMap;

import static dev.birb.wgpu.WgpuMcMod.LOGGER;

public class Wgpu {
    private static final WgpuTextureManager textureManager = new WgpuTextureManager();

    public volatile static boolean INITIALIZED = false;
    public volatile static boolean MAY_INITIALIZE = false;

    public static HashMap<String, Integer> blocks;
    public static String wmIdentity;

    public static HashMap<Integer, Integer> keyState = new HashMap<>();

    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }
    public static HashMap<Integer, Integer> keyStates = new HashMap<>();
    public static void preInit(String windowTitle) {
        try {
            WgpuNative.load("wgpu_mc_jni", true);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }

        WgpuNative.preInit();
    }

    public static void startRendering() {
        if (!INITIALIZED) {
            try {
                System.loadLibrary("renderdoc");
            } catch(UnsatisfiedLinkError e) {
                e.printStackTrace();
            }

            WgpuNative.startRendering("Minecraft");
        } else {
            throw new RuntimeException("wgpu-mc has already been initialized");
        }
    }

    public static void mouseMove(double x, double y) {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> {
            client.mouse.onCursorPos(0, x, y);
        });
    }

    public static void mouseAction(int button, int action) {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> client.mouse.onMouseButton(-1, button, action, 0));
    }
    public static void onChar(int codepoint) {
        MinecraftClient client = MinecraftClient.getInstance();
        //TODO: Pull real modifiers
        int modifiers = 0;
        client.execute(() -> client.keyboard.onChar(0,codepoint,modifiers));

    }

    public static void keyState(int key, int scancode, int state, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        int converted = Wgpu.convertKeyCode(key);
        keyStates.put(converted, state);
        /// Old debugging stuff, might be useful to keep around
        // System.out.println(String.format("Put Key %s (scan %s conv %s) to state %s", key, scancode, converted, state));

        client.execute(() -> {
            Wgpu.keyState.put(key, state);
            client.keyboard.onKey(0, key, scancode, state, modifiers);
        });
    }

    public static void onResize() {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> MinecraftClient.getInstance().onResolutionChanged());
    }

    public static void render(MinecraftClient client) {
        WgpuNative.setWorldRenderState(client.world != null);
    }
    public static int convertKeyCode(int code) {
        int converted = -1;

        if (code >= 10 && code <= 35) {
            // winit lowercase alphabet starts at 10
            // GLFW  uppercase alphabet starts at 65 (+55 from 10), lowercase 32 chars later.
            return code + 55 + 32;
        }
        switch (code) {
            case 118 -> converted = GLFW.GLFW_KEY_LEFT_SHIFT;
            case 139 -> converted = GLFW.GLFW_KEY_RIGHT_SHIFT;
            case 117 -> converted = GLFW.GLFW_KEY_LEFT_CONTROL;
            case 138 -> converted = GLFW.GLFW_KEY_RIGHT_CONTROL;
            case 39 -> converted = GLFW.GLFW_KEY_F3;
            case 74 -> converted = GLFW.GLFW_KEY_BACKSPACE;
        }
        System.out.printf("Couldn't convert %s\n", code);
        return converted;
    }

}
