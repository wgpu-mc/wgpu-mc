package dev.birb.wgpu.rust;

import lombok.Getter;
import lombok.Setter;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

import java.io.IOException;
import java.util.NoSuchElementException;

public class WgpuResourceProvider {

    @Getter
    @Setter
    private static ResourceManager manager;

    public static byte[] getResource(String path) {
        try {
            return WgpuNative.digestInputStream(
                    manager.getResource(new Identifier(path)).orElseThrow().getInputStream()
            );
        } catch (IOException | NoSuchElementException e) {
            return new byte[0];
        }
    }

}
