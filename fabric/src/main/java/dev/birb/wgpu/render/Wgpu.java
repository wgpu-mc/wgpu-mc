package dev.birb.wgpu.render;

import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.block.Block;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;

import java.lang.reflect.InvocationTargetException;
import java.util.HashMap;
import java.util.concurrent.atomic.AtomicReference;

public class Wgpu {
    private static final WgpuTextureManager textureManager = new WgpuTextureManager();
    public volatile static boolean INITIALIZED = false;
    public volatile static boolean MAY_INITIALIZE = false;

    public static HashMap<String, Integer> blocks;

    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }

    public static void preInit(String windowTitle) {
        try {
            WgpuNative.load("wgpu_mc_jni", true);
        } catch (Throwable e) {
            e.printStackTrace();
            System.exit(1);
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
            client.mouse.x = x;
            client.mouse.y = y;
        });

        client.mouse.x = x;
        client.mouse.y = y;
    }

    public static void mouseAction(int button, int action) {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> client.mouse.onMouseButton(-1, button, action, 0));
    }

    public static void onResize() {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> MinecraftClient.getInstance().onResolutionChanged());
    }

    public static void render(MinecraftClient client) {
        WgpuNative.setWorldRenderState(client.world != null);
    }
}
