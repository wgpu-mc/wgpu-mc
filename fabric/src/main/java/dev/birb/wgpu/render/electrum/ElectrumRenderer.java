package dev.birb.wgpu.render.electrum;

import net.fabricmc.fabric.api.renderer.v1.Renderer;
import net.fabricmc.fabric.api.renderer.v1.material.MaterialFinder;
import net.fabricmc.fabric.api.renderer.v1.material.RenderMaterial;
import net.fabricmc.fabric.api.renderer.v1.mesh.MeshBuilder;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.Nullable;

public class ElectrumRenderer implements Renderer {

    @Override
    public MeshBuilder meshBuilder() {
        return null;
    }

    @Override
    public MaterialFinder materialFinder() {
        return null;
    }

    @Override
    public @Nullable RenderMaterial materialById(Identifier id) {
        return null;
    }

    @Override
    public boolean registerMaterial(Identifier id, RenderMaterial material) {
        return false;
    }

}
