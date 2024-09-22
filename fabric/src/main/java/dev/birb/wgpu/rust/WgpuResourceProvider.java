package dev.birb.wgpu.rust;

import dev.birb.wgpu.WgpuMcMod;
import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

public class WgpuResourceProvider {

    public static ResourceManager manager;

    public static byte[] getResource(String path) {
        if(manager != null) {
            try {
                return manager.getResourceOrThrow(new Identifier(path)).getInputStream().readAllBytes();
            } catch (Exception e) {
                WgpuMcMod.LOGGER.error(e.getMessage());
                return new byte[0];
            }
        }

        try {
            return MinecraftClient.getInstance().getResourceManager().getResourceOrThrow(new Identifier(path)).getInputStream().readAllBytes();
        } catch (Exception e) {
            WgpuMcMod.LOGGER.error(e.getMessage());
            return new byte[0];
        }
    }
}
