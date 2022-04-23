package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(ClientPlayNetworkHandler.class)
public class ClientPlayNetworkHandlerMixin {

    /**
     * @author wgpu-mc
     * @reason tell the rust backend to rebuild the chunk mesh
     */
    @Overwrite
    public void scheduleRenderChunk(WorldChunk chunk, int x, int z) {
       WgpuNative.scheduleChunkRebuild(x, z);
    }

}
