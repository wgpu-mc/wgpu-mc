package dev.birb.wgpu.rust;

import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

public class WgpuResourceProvider {

    public static ResourceManager manager;

    public static byte[] getResource(String path) {
        if(manager != null) {
            try {
                return manager.getResourceOrThrow(Identifier.of(path)).getInputStream().readAllBytes();
            } catch (Exception e) {
                return new byte[0];
            }
        }

        try {
            return MinecraftClient.getInstance().getResourceManager().getResourceOrThrow(Identifier.of(path)).getInputStream().readAllBytes();
        } catch (Exception e) {
            return new byte[0];
        }
    }
}
