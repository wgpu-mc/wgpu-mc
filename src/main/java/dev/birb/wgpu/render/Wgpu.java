package dev.birb.wgpu.render;

import dev.birb.wgpu.game.MainGameThread;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.block.Block;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.HashMap;

public class Wgpu {
    public static boolean INITIALIZED = false;

    private static final WgpuTextureManager textureManager = new WgpuTextureManager();
    public static HashMap<String, Integer> blocks;

    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }

    public static void preInit(String windowTitle) {
//        try {
//            NativeUtils.loadLibraryFromJar("assets/wgpu_mc/dylib/libwgpu_mc_jni.dylib");
//            System.out.println("Loaded dylib");
//        } catch (IOException e) {
//            e.printStackTrace();
//            throw new UncheckedIOException(e);
//        }
        try {
            System.load("/Users/birb/wgpu-mc/target/debug/libwgpu_mc_jni.dylib");
        } catch(Throwable e) {
            e.printStackTrace();
            throw new RuntimeException(e);
        }

        WgpuNative.initialize(windowTitle);

    }

    public static void initRenderer(MinecraftClient client) {
        if(!INITIALIZED) {
            WgpuNative.initRenderer();
            INITIALIZED = true;

            for(Block block : Registry.BLOCK) {
                Identifier blockId = Registry.BLOCK.getId(block);
                WgpuNative.registerEntry(0, "minecraft:" + blockId.getPath());
            }

            blocks = WgpuNative.bakeBlockModels();

            client.updateWindowTitle();
            MainGameThread.createNewThread(client);
            WgpuNative.doEventLoop();
        }
    }

    public static void render(MinecraftClient client) {
        WgpuNative.setWorldRenderState(client.world != null);
    }

}
