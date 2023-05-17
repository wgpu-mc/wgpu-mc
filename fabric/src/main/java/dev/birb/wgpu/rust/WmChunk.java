package dev.birb.wgpu.rust;

import dev.birb.wgpu.mixin.accessors.PackedIntegerArrayAccessor;
import dev.birb.wgpu.palette.RustPalette;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import net.minecraft.client.MinecraftClient;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PalettedContainer;
import net.minecraft.world.chunk.WorldChunk;

public class WmChunk {
    public WorldChunk worldChunk;
    public int x;
    public int z;

    public WmChunk(WorldChunk worldChunk) {
        MinecraftClient client = MinecraftClient.getInstance();

        this.x = worldChunk.getPos().x;
        this.z = worldChunk.getPos().z;

        this.worldChunk = worldChunk;
    }

    public void uploadAndBake() throws ClassCastException {
        long[] paletteIndices = new long[24];
        long[] storageIndices = new long[24];

        assert this.worldChunk.getSectionArray().length == 24;

        for(int i=0;i<24;i++) {
//            RustPalette<?> rustPalette = (RustPalette<?>) this.worldChunk.getSection(i).getBlockStateContainer().data.palette;;
            Palette<?> palette;
            PalettedContainer<?> container;
            try {
                palette = this.worldChunk.getSection(i).getBlockStateContainer().data.palette;
                container = this.worldChunk.getSection(i).getBlockStateContainer();
            } catch(ArrayIndexOutOfBoundsException e) {
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

            if(paletteStorage instanceof PackedIntegerArrayAccessor accessor) {
                long index = WgpuNative.createPaletteStorage(
                    paletteStorage.getData(),
                    accessor.getElementsPerLong(),
                    paletteStorage.getElementBits(),
                    accessor.getMaxValue(),
                    accessor.getIndexScale(),
                    accessor.getIndexOffset(),
                    accessor.getIndexShift(),
                    paletteStorage.getSize()
                );

                storageIndices[i] = index + 1;
            }
        }

        int x = this.x;
        int z = this.z;

        Thread thread = new Thread(() -> {
            WgpuNative.createChunk(x, z, paletteIndices, storageIndices);
            WgpuNative.bakeChunk(x, z);
        });

        thread.start();
    }
}
