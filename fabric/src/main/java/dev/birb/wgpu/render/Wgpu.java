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

    public static void render(MinecraftClient client) {
        WgpuNative.setWorldRenderState(client.world != null);
    }
}
