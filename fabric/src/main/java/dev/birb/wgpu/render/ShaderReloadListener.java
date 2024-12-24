package dev.birb.wgpu.render;

import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuResourceProvider;
import net.fabricmc.fabric.api.resource.SimpleSynchronousResourceReloadListener;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

public class ShaderReloadListener implements SimpleSynchronousResourceReloadListener {
    @Override
    public Identifier getFabricId() {
        return Identifier.of("electrum", "listener");
    }

    @Override
    public void reload(ResourceManager manager) {
        WgpuResourceProvider.manager = manager;
        WgpuNative.reloadShaders();
    }
}
