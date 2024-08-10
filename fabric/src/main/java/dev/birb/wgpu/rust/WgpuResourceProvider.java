package dev.birb.wgpu.rust;

import net.minecraft.client.MinecraftClient;
import net.minecraft.util.Identifier;

public class WgpuResourceProvider {

    
    public static byte[] getResource(String path) {
        try {
            return MinecraftClient.getInstance().getResourceManager().getResourceOrThrow(new Identifier(path)).getInputStream().readAllBytes();
        } catch (Exception e) {
            return new byte[0];
        }
    }
}
