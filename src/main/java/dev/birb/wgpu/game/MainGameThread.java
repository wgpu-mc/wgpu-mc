package dev.birb.wgpu.game;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.ConnectScreen;
import net.minecraft.client.network.ServerInfo;

public class MainGameThread extends Thread {

    public static void createNewThread(MinecraftClient client) {
        Thread gameThread = new Thread() {
            @Override
            public void run() {
                client.openScreen(new ConnectScreen(null, client, new ServerInfo("Test", "localhost:25565", false)));
                client.run();
            }
        };

        gameThread.setName("Run loop, diverted");
        gameThread.start();
    }

}
