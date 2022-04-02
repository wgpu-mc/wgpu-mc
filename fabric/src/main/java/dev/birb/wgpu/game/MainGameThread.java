package dev.birb.wgpu.game;

import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.ConnectScreen;
import net.minecraft.client.network.ServerInfo;

public class MainGameThread extends Thread {

    public static void createNewThread(MinecraftClient client) {
        Thread gameThread = new Thread() {
            @Override
            public void run() {
//                client.startIntegratedServer("New World (1)");
                client.run();
            }
        };

        gameThread.setName("Run loop, diverted");
//        RenderSystem.gameThread = gameThread;
//        RenderSystem.renderThread = Thread.currentThread();
        gameThread.start();
    }

}
