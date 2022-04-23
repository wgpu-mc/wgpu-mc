package dev.birb.wgpu.render;

import dev.birb.wgpu.mixin.core.KeyboardMixin;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.client.MinecraftClient;
import org.lwjgl.glfw.GLFW;

import java.util.HashMap;

import static dev.birb.wgpu.WgpuMcMod.LOGGER;
import static dev.birb.wgpu.input.WgpuKeys.*;

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
    public static void onChar(int codepoint, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        int mappedModifier = convertModifiers(modifiers);
       System.out.printf("onChar(%s, %s)\n", codepoint, modifiers);
       System.out.printf("Unmapped Shift: %s, Ctrl: %s, Alt: %s, Super: %s\n", modifiers & GLFW.GLFW_MOD_SHIFT, modifiers & GLFW.GLFW_MOD_CONTROL, modifiers & GLFW.GLFW_MOD_ALT, modifiers & GLFW.GLFW_MOD_SUPER);
       System.out.printf("Mapped   Shift: %s, Ctrl: %s, Alt: %s, Super: %s\n", mappedModifier & GLFW.GLFW_MOD_SHIFT, mappedModifier & GLFW.GLFW_MOD_CONTROL, mappedModifier & GLFW.GLFW_MOD_ALT, mappedModifier & GLFW.GLFW_MOD_SUPER);

        client.execute(() -> client.keyboard.onChar(0,codepoint,mappedModifier));

    }

    public static void keyState(int key, int scancode, int state, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        int convertedKey = convertKeyCode(key);
        int convertedModifier = convertModifiers(modifiers);
        keyStates.put(convertedKey, state);
        /// Old debugging stuff, might be useful to keep around
        // System.out.println(String.format("Put Key %s (scan %s conv %s) to state %s", key, scancode, converted, state));

        client.execute(() -> {
            Wgpu.keyState.put(key, state);
            client.keyboard.onKey(0, key, scancode, state, convertedModifier);
        });
    }

    public static void onResize() {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> MinecraftClient.getInstance().onResolutionChanged());
    }

    public static void render(MinecraftClient client) {
        WgpuNative.setWorldRenderState(client.world != null);
    }


}
