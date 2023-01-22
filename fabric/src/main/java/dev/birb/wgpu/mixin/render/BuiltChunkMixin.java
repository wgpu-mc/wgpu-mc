package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.rust.WmChunk;
import net.minecraft.client.render.RenderLayer;
import net.minecraft.client.render.chunk.ChunkBuilder;
import net.minecraft.client.render.chunk.ChunkRendererRegionBuilder;
import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

@Mixin(ChunkBuilder.BuiltChunk.class)
public class BuiltChunkMixin {

    /**
     * @author wgpu-mc
     * @reason we do this in Rust
     */
    @Overwrite
    public void rebuild(ChunkRendererRegionBuilder builder) {
        ((ChunkBuilder.BuiltChunk) (Object) this).createRebuildTask(builder);
    }

    /**
     * @author wgpu-mc
     * @reason Rust builds the chunks
     */
    @Inject(method = "createRebuildTask", cancellable = true, at = @At("RETURN"), locals = LocalCapture.CAPTURE_FAILHARD)
    public void createRebuildTask(ChunkRendererRegionBuilder builder, CallbackInfoReturnable<ChunkBuilder.BuiltChunk.Task> cir) {
        for(ChunkRendererRegionBuilder.ClientChunk chunk : builder.chunks.values()) {
            WorldChunk worldChunk = chunk.getChunk();
            WmChunk wmChunk = new WmChunk(worldChunk);
            wmChunk.uploadAndBake();
        }
        cir.setReturnValue(null);
    }

    /**
     * @author wgpu-mc
     * @reason Rust builds the chunks
     */
    @Overwrite
    public void scheduleRebuild(ChunkBuilder chunkRenderer, ChunkRendererRegionBuilder builder) {
        ((ChunkBuilder.BuiltChunk) (Object) this).createRebuildTask(builder);
    }

    /**
     * @author wgpu-mc
     * @reason N/A
     */
    @Overwrite
    public boolean scheduleSort(RenderLayer layer, ChunkBuilder chunkRenderer) {
        return true;
    }

}
