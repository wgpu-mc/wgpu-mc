package dev.birb.wgpu.rust;

import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.palette.RustPalette;
import dev.birb.wgpu.render.Wgpu;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.math.ChunkSectionPos;
import net.minecraft.world.chunk.ChunkNibbleArray;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PalettedContainer;
import net.minecraft.world.chunk.WorldChunk;
import net.minecraft.world.chunk.light.ChunkLightProvider;
import net.minecraft.world.chunk.light.SkyLightStorage;
import org.lwjgl.BufferUtils;

import java.nio.ByteBuffer;

public class WmChunk {
    private final WorldChunk worldChunk;
    private final int x;
    private final int z;

    public WmChunk(WorldChunk worldChunk) {
        this.x = worldChunk.getPos().x;
        this.z = worldChunk.getPos().z;

        this.worldChunk = worldChunk;
    }

    public void uploadAndBake() throws ClassCastException {
        long[] paletteIndices = new long[24];
        long[] storageIndices = new long[24];

        assert this.worldChunk.getSectionArray().length == 24;

        ChunkLightProvider<?, ?> skyLightProvider = worldChunk.getWorld().getLightingProvider().skyLightProvider;
        ChunkLightProvider<?, ?> blockLightProvider = worldChunk.getWorld().getLightingProvider().blockLightProvider;

        ByteBuffer skyBytes = ByteBuffer.allocateDirect(2048 * 24);
        ByteBuffer blockBytes = ByteBuffer.allocateDirect(2048 * 24);

        ClientPlayerEntity player = MinecraftClient.getInstance().player;
        ChunkPos pos = player.getChunkPos();

        if(this.worldChunk.getPos().equals(pos)) {
            int blockLight = this.worldChunk.getWorld().getLightLevel(player.getBlockPos());
            int a = blockLight;
        }

        for (int i = 0; i < 24; i++) {
            Palette<?> palette;
            PalettedContainer<?> container;
            try {
                palette = this.worldChunk.getSection(i).getBlockStateContainer().data.palette;
                container = this.worldChunk.getSection(i).getBlockStateContainer();
            } catch (ArrayIndexOutOfBoundsException e) {
                continue;
            }

            BlockPos chunkBlockPos = this.worldChunk.getPos().getBlockPos(0, (i * 16) - 64, 0);
            long sectionPos = ChunkSectionPos.fromBlockPos(chunkBlockPos.asLong());

            if(skyLightProvider != null && blockLightProvider != null) {
                ChunkNibbleArray skyNibble = skyLightProvider.lightStorage.uncachedStorage.get(sectionPos);
                ChunkNibbleArray blockNibble = blockLightProvider.lightStorage.uncachedStorage.get(sectionPos);

                if(skyNibble != null) {
                    byte[] sectionSkyLightBytes = skyNibble.asByteArray();
                    skyBytes.put(i * 2048, sectionSkyLightBytes);
                }

                if(blockNibble != null) {
                    byte[] sectionBlockLightBytes = blockNibble.asByteArray();
                    blockBytes.put(i * 2048, sectionBlockLightBytes);
                }
            }

            PaletteStorage paletteStorage = container.data.storage;

            RustPalette rustPalette = new RustPalette(
                    container.idList,
                    WgpuNative.uploadIdList((IndexedIterable<Object>) container.idList)
            );

            ByteBuf buf = Unpooled.buffer(palette.getPacketSize());
            PacketByteBuf packetBuf = new PacketByteBuf(buf);
            palette.writePacket(packetBuf);
            rustPalette.readPacket(packetBuf);

            paletteIndices[i] = rustPalette.getSlabIndex() + 1;

            if (paletteStorage instanceof PackedIntegerArray array) {
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

                storageIndices[i] = index + 1;
            }
        }

        byte[] skyLightDebug = new byte[24 * 2048];
        skyBytes.get(skyLightDebug);

        Thread thread = new Thread(() -> {
            WgpuNative.createChunk(this.x, this.z, paletteIndices, storageIndices, blockBytes, skyBytes);
            WgpuNative.bakeChunk(this.x, this.z);
        });

        thread.start();
    }
}
