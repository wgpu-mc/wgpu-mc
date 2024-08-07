package dev.birb.wgpu.rust;

import lombok.Getter;
import lombok.Setter;
import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.Resource;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

import java.io.IOException;
import java.util.NoSuchElementException;
import java.util.Optional;

public class WgpuResourceProvider {
    public static byte[] getResource(String path) {
        System.out.println("!!!getResource");
        try {
            return MinecraftClient.getInstance().getResourceManager().getResource(new Identifier(path)).orElseThrow().getInputStream().readAllBytes();
        } catch (Exception e) {
            System.out.println(e.getMessage());
            return new byte[0];
        }
    }
}
