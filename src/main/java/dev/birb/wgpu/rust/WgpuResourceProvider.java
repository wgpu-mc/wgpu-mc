package dev.birb.wgpu.rust;

import net.minecraft.client.MinecraftClient;
import net.minecraft.util.Identifier;

import java.io.IOException;

public class WgpuResourceProvider {

    public static byte[] getResource(String namespace, String path) throws IOException {
        try {
            return WgpuNative.digestInputStream(
                MinecraftClient.getInstance()
                    .getResourceManager()
                    .getResource(new Identifier(namespace, path))
                    .getInputStream()
            );
        } catch(IOException e) {
            e.printStackTrace();
            throw e;
        }
    }

}
