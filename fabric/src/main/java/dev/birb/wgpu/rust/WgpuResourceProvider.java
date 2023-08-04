package dev.birb.wgpu.rust;

import lombok.Getter;
import lombok.Setter;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

import java.io.IOException;

public class WgpuResourceProvider {

    @Getter
    @Setter
    private static ResourceManager manager;

    public static byte[] getResource(String path) {
        try {
            return WgpuNative.digestInputStream(
                manager.getResource(new Identifier(path)).getInputStream()
            );
        } catch(IOException e) {
            return new byte[0];
        }
    }

}
