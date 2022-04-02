package dev.birb.wgpu.render;

import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.block.Block;
import net.minecraft.client.MinecraftClient;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;

import java.util.HashMap;

public class Wgpu {
    private static final WgpuTextureManager textureManager = new WgpuTextureManager();
    public static boolean INITIALIZED = false;
    public static boolean MAY_INITIALIZE = false;

    public static HashMap<String, Integer> blocks;

    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }

    public static void preInit(String windowTitle) {
        try {
            WgpuNative.load("wgpu_mc_jni", true);
        } catch (Throwable e) {
            e.printStackTrace();
            throw new RuntimeException(e);
        }

        WgpuNative.initialize(windowTitle);
    }

    public static void initRenderer() {
        if (!INITIALIZED) {
            System.loadLibrary("renderdoc");

            System.out.println("Initializing wgpu-mc renderer");
            WgpuNative.initRenderer();
            INITIALIZED = true;

            for (Block block : Registry.BLOCK) {
                Identifier blockId = Registry.BLOCK.getId(block);
                WgpuNative.registerEntry(0, "minecraft:" + blockId.getPath());
            }

            blocks = WgpuNative.bakeBlockModels();

            WgpuNative.doEventLoop();
        }
    }

    public static void render(MinecraftClient client) {
        WgpuNative.setWorldRenderState(client.world != null);
    }
}
