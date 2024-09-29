package dev.birb.wgpu.render;

import net.minecraft.client.render.chunk.ChunkBuilder;

public interface RebuildTaskAccessor {

    void wgpu_mc$setBuiltChunk(ChunkBuilder.BuiltChunk builtChunk);

}
