package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.palette.RustPalette;
import dev.birb.wgpu.render.RebuildTaskAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.chunk.BlockBufferBuilderStorage;
import net.minecraft.client.render.chunk.ChunkBuilder;
import net.minecraft.client.render.chunk.ChunkRendererRegion;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkSectionPos;
import net.minecraft.util.math.Vec3i;
import net.minecraft.world.chunk.*;
import net.minecraft.world.chunk.light.ChunkLightProvider;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.util.Arrays;
import java.util.concurrent.CompletableFuture;

@Mixin(targets = "net/minecraft/client/render/chunk/ChunkBuilder$BuiltChunk$RebuildTask")
public class RebuildTaskMixin implements RebuildTaskAccessor {

    @Shadow @Nullable protected ChunkRendererRegion region;

    private ChunkBuilder.BuiltChunk builtChunk;

    /**
     * @author wgpu-mc
     * @reason Build chunks in Rust
     */
    @Overwrite
    public CompletableFuture<ChunkBuilder.Result> run(BlockBufferBuilderStorage buffers) {
        ChunkBuilder.BuiltChunk chunk = this.builtChunk;

        if (((ChunkBuilder.BuiltChunk.Task) (Object) this).cancelled.get()) {
            return CompletableFuture.completedFuture(ChunkBuilder.Result.CANCELLED);
        } else if (!chunk.shouldBuild()) {
            this.region = null;
            chunk.scheduleRebuild(false);
            ((ChunkBuilder.BuiltChunk.Task) (Object) this).cancelled.set(true);
            return CompletableFuture.completedFuture(ChunkBuilder.Result.CANCELLED);
        } else if (((ChunkBuilder.BuiltChunk.Task) (Object) this).cancelled.get()) {
            return CompletableFuture.completedFuture(ChunkBuilder.Result.CANCELLED);
        } else {
            long[] paletteIndices = new long[27];
            long[] storageIndices = new long[27];
            Arrays.fill(paletteIndices,-1);
            Arrays.fill(storageIndices,-1);
            ClientWorld world = MinecraftClient.getInstance().world;

            MinecraftClient client = MinecraftClient.getInstance();
            ChunkLightProvider<?, ?> skyLightProvider = world.getLightingProvider().skyLightProvider;
            ChunkLightProvider<?, ?> blockLightProvider = world.getLightingProvider().blockLightProvider;

            byte[][] skyIndices = new byte[27][2048];
            byte[][] blockIndices = new byte[27][2048];
            BlockPos origin = chunk.getOrigin();
            Vec3i sectionCoord = new Vec3i(origin.getX()>>4,origin.getY()>>4,origin.getZ()>>4);
            for(int x=0;x<3;x++){
                for(int z=0;z<3;z++){
                    WorldChunk worldChunk = (WorldChunk)world.getChunk(sectionCoord.getX()+x-1, sectionCoord.getZ()+z-1, ChunkStatus.FULL,false);
                    if(worldChunk==null)continue;
                    for(int y=0;y<3;y++){
                        int id = x+3*y+9*z;
                        Palette<?> palette;
                        PalettedContainer<?> section;
                        try {
                            section = worldChunk.getSection(world.sectionCoordToIndex(sectionCoord.getY()+y-1)).getBlockStateContainer();
                            palette = section.data.palette;
                        } catch (ArrayIndexOutOfBoundsException e) {
                            continue;
                        }

                        long sectionPos = ChunkSectionPos.from(sectionCoord.getX()+x-1,sectionCoord.getY()+y-1,sectionCoord.getZ()+z-1).asLong();
                        if(skyLightProvider != null && blockLightProvider != null) {
                            ChunkNibbleArray skyNibble = skyLightProvider.lightStorage.uncachedStorage.get(sectionPos);
                            ChunkNibbleArray blockNibble = blockLightProvider.lightStorage.uncachedStorage.get(sectionPos);
                            if(skyNibble != null) {
                                skyIndices[id]=skyNibble.asByteArray();
                            }
                            if(blockNibble != null) {
                                blockIndices[id]=blockNibble.asByteArray();
                            }
                        }

                        PaletteStorage paletteStorage = section.data.storage;

                        if (paletteStorage instanceof PackedIntegerArray array) {
                            //palette
                            RustPalette rustPalette = new RustPalette(section.idList);

                            ByteBuf buf = Unpooled.buffer(palette.getPacketSize());
                            PacketByteBuf packetBuf = new PacketByteBuf(buf);
                            if(palette.getSize() == 1){
                                packetBuf.writeInt(1);
                            }
                            palette.writePacket(packetBuf);
                            rustPalette.readPacket(packetBuf);

                            paletteIndices[id] = rustPalette.getSlabIndex();

                            //PackedIntegerArray
                            long index = WgpuNative.createPaletteStorage(
                                    paletteStorage.getData(),
                                    array.elementsPerLong,
                                    paletteStorage.getElementBits(),
                                    array.maxValue,
                                    array.indexScale,
                                    array.indexOffset,
                                    array.indexShift,
                                    paletteStorage.getSize()
                            );

                            storageIndices[id] = index;
                        }
                    }
                }
            }
            WgpuNative.bakeSection(sectionCoord.getX(),sectionCoord.getY(),sectionCoord.getZ(),paletteIndices, storageIndices, blockIndices, skyIndices);
            return CompletableFuture.completedFuture(ChunkBuilder.Result.SUCCESSFUL);
        }
    }

    @Override
    public void wgpu_mc$setBuiltChunk(ChunkBuilder.BuiltChunk builtChunk) {
        this.builtChunk = builtChunk;
    }
}
