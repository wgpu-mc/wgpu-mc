package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.network.packet.s2c.play.ChunkDeltaUpdateS2CPacket;
import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientPlayNetworkHandler.class)
public class ClientPlayNetworkHandlerMixin {

//    /**
//     * @author wgpu-mc
//     * @reason tell the rust backend to rebuild the chunk mesh
//     */?
//    @Overwrite
//    public void scheduleRenderChunk(WorldChunk chunk, int x, int z) {
//       WgpuNative.scheduleChunkRebuild(x, z);
//    }

    public void otherNothing() {

    }

    @Inject(method="onChunkDeltaUpdate", at = @At("HEAD"))
    public void doNothing(ChunkDeltaUpdateS2CPacket packet, CallbackInfo ci) {
        otherNothing();
    }

}
