package dev.birb.wgpu.render;

import dev.birb.wgpu.game.MainGameThread;
import dev.birb.wgpu.mixin.accessors.ScreenAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import net.minecraft.block.Block;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.AbstractButtonWidget;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;

import java.util.HashMap;
import java.util.List;

public class Wgpu {
    public static boolean INITIALIZED = false;

    private static final WgpuTextureManager textureManager = new WgpuTextureManager();
    public static HashMap<String, Integer> blocks;

    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }

    public static void preInit(String windowTitle) {
        System.load("/Users/birb/wgpu-mc/target/debug/libwgpu_mc_jni.dylib");
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
