package dev.birb.wgpu.render;

import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.block.Block;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.entity.vehicle.MinecartEntity;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;

import java.lang.reflect.InvocationTargetException;
import java.util.HashMap;
import java.util.concurrent.atomic.AtomicReference;

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

    public static void keyState(int key, int scancode, int state, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();

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
}
