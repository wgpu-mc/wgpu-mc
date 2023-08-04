package dev.birb.wgpu.rust;

import dev.birb.wgpu.palette.RustPalette;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PalettedContainer;
import net.minecraft.world.chunk.WorldChunk;

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

        for (int i = 0; i < 24; i++) {
            Palette<?> palette;
            PalettedContainer<?> container;
            try {
                palette = this.worldChunk.getSection(i).getBlockStateContainer().data.palette;
                container = this.worldChunk.getSection(i).getBlockStateContainer();
            } catch (ArrayIndexOutOfBoundsException e) {
                continue;
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

        Thread thread = new Thread(() -> {
            WgpuNative.createChunk(this.x, this.z, paletteIndices, storageIndices);
            WgpuNative.bakeChunk(this.x, this.z);
        });

        thread.start();
    }
}
