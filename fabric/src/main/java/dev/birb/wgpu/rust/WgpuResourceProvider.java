package dev.birb.wgpu.rust;

import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

import java.io.IOException;

public class WgpuResourceProvider {

    public static ResourceManager manager;

    public static byte[] getResource(String namespace, String path) {
        try {
            return WgpuNative.digestInputStream(
                manager.getResource(new Identifier(namespace, path)).getInputStream()
            );
        } catch(IOException e) {
            return null;
        }
    }

}
