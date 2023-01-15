package dev.birb.wgpu.render;

import dev.birb.wgpu.entity.EntityState;
import dev.birb.wgpu.palette.RustBlockStateAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.client.MinecraftClient;

import org.lwjgl.glfw.GLFW;
import sun.misc.Unsafe;

import java.lang.reflect.Field;
import java.util.HashMap;

import static dev.birb.wgpu.WgpuMcMod.LOGGER;
import static dev.birb.wgpu.input.WgpuKeys.*;

public class Wgpu {
    private static final WgpuTextureManager textureManager = new WgpuTextureManager();

    public volatile static boolean INITIALIZED = false;
    public volatile static boolean MAY_INITIALIZE = false;

    public volatile static RuntimeException EXCEPTION;

    public static HashMap<String, Integer> blocks;
    public static String wmIdentity;
    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }
    public static HashMap<Integer, Integer> keyStates = new HashMap<>();

    public static Unsafe UNSAFE;

    public static int windowWidth = 1280;
    public static int windowHeight = 720;
    static {
        Field f = null; //Internal reference
        try {
            f = Unsafe.class.getDeclaredField("theUnsafe");
            f.setAccessible(true);
            UNSAFE = (Unsafe) f.get(null);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public static void linkRenderDoc() {
        try {
            System.loadLibrary("renderdoc");
        } catch(UnsatisfiedLinkError e) {
            LOGGER.debug("Error while loading RenderDoc:\n" + e.getMessage());
            e.printStackTrace();
        }
    }


    public static void startRendering() {
        if (!INITIALIZED) {
            linkRenderDoc();
            WgpuNative.startRendering("Minecraft");
        } else {
            throw new RuntimeException("wgpu-mc has already been initialized");
        }
    }

    public static void cursorMove(double x, double y) {
        MinecraftClient client = net.minecraft.client.MinecraftClient.getInstance();

        client.execute(() -> {
//            if(client.isWindowFocused() && !client.mouse.isCursorLocked()) {
                client.mouse.onCursorPos(0, x, y);
//            }
        });
    }

    public static void mouseMove(double x, double y) {
//        MinecraftClient client = net.minecraft.client.MinecraftClient.getInstance();
//
//        client.execute(() -> {
//            if(client.isWindowFocused() && client.mouse.isCursorLocked()) {
//                client.mouse.cursorDeltaX = x;
//                client.mouse.cursorDeltaY = y;
//                client.mouse.updateMouse();
//            }
//        });
    }

    public static void mouseAction(int button, int action) {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> client.mouse.onMouseButton(-1, button, action, 0));
    }

    public static void onChar(int codepoint, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        int mappedModifier = convertModifiers(modifiers);
//       System.out.printf("onChar(%s, %s)\n", codepoint, modifiers);
//       System.out.printf("Unmapped Shift: %s, Ctrl: %s, Alt: %s, Super: %s\n", modifiers & GLFW.GLFW_MOD_SHIFT, modifiers & GLFW.GLFW_MOD_CONTROL, modifiers & GLFW.GLFW_MOD_ALT, modifiers & GLFW.GLFW_MOD_SUPER);
//       System.out.printf("Mapped   Shift: %s, Ctrl: %s, Alt: %s, Super: %s\n", mappedModifier & GLFW.GLFW_MOD_SHIFT, mappedModifier & GLFW.GLFW_MOD_CONTROL, mappedModifier & GLFW.GLFW_MOD_ALT, mappedModifier & GLFW.GLFW_MOD_SUPER);

        client.execute(() -> client.keyboard.onChar(0,codepoint,mappedModifier));

    }

    public static void keyState(int key, int scancode, int state, int modifiers) {

        MinecraftClient client = MinecraftClient.getInstance();
        int convertedKey = convertKeyCode(key);
        int convertedModifier = convertModifiers(modifiers);
        int convertedState = state == 0 ? GLFW.GLFW_PRESS : GLFW.GLFW_RELEASE;
//        System.out.printf("keyState(%s:%s, %s, %s, %s)\n", key, convertedKey, scancode, state, modifiers);
//        System.out.printf("Unmapped Shift: %s, Ctrl: %s, Alt: %s, Super: %s\n", modifiers & GLFW.GLFW_MOD_SHIFT, modifiers & GLFW.GLFW_MOD_CONTROL, modifiers & GLFW.GLFW_MOD_ALT, modifiers & GLFW.GLFW_MOD_SUPER);
//        System.out.printf("Mapped   Shift: %s, Ctrl: %s, Alt: %s, Super: %s\n", convertedModifier & GLFW.GLFW_MOD_SHIFT, convertedModifier & GLFW.GLFW_MOD_CONTROL, convertedModifier & GLFW.GLFW_MOD_ALT, convertedModifier & GLFW.GLFW_MOD_SUPER);
        Wgpu.keyStates.put(convertedKey, state);
//        System.out.printf("set keystates[%s]=%s\n", convertedKey, Wgpu.keyStates);

        client.execute(() -> client.keyboard.onKey(0, convertedKey, scancode, convertedState, convertedModifier));
    }

    public static void onResize(int width, int height) {
        Wgpu.windowWidth = width;
        Wgpu.windowHeight = height;
        MinecraftClient client = MinecraftClient.getInstance();
        client.execute(client::onResolutionChanged);
    }

    public static void rustPanic(String message) {
        EXCEPTION = new RuntimeException(message);
        LOGGER.error(message);
        while(true) {}
    }

    public static void helperSetBlockStateIndex(Object o, int blockstateKey) {
        ((RustBlockStateAccessor) o).setRustBlockStateIndex(blockstateKey);
    }

    public static void helperSetPartIndex(String entity, String part, int index) {
        if(!EntityState.matrixIndices.containsKey(entity)) {
            EntityState.matrixIndices.put(entity, new HashMap<>());
        }

        EntityState.matrixIndices.get(entity).put(part, index);
    }

    public static void debug(Object o) {
        // MinecraftClient.getInstance().inGameHud.addChatMessage(MessageType.CHAT, new LiteralText(o.toString()), UUID.randomUUID());
        System.out.println(o);
    }

    public static void windowFocused(boolean focused) {
        MinecraftClient.getInstance().onWindowFocusChanged(focused);
    }

}
