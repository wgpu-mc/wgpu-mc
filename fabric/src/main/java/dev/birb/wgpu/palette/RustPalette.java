package dev.birb.wgpu.palette;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PaletteResizeListener;

import java.lang.ref.Cleaner;
import java.nio.ByteOrder;
import java.util.List;
import java.util.function.Predicate;

public class RustPalette {

    private final long slabIndex;
    private final IndexedIterable<?> idList;

    public RustPalette(IndexedIterable<?> idList, long rustIdList) {
        this.idList = idList;
        this.slabIndex = WgpuNative.createPalette(rustIdList);
    }

    public void readPacket(PacketByteBuf buf) {
        int index = buf.readerIndex();
        int size = buf.readVarInt();

        int[] blockstateOffsets = new int[size];

        for(int i=0;i<size;i++) {
            Object object = this.idList.get(buf.readVarInt());
            RustBlockStateAccessor accessor = (RustBlockStateAccessor) object;
            blockstateOffsets[i] = accessor.getRustBlockStateIndex();
        }

        WgpuNative.paletteReadPacket(this.slabIndex, buf.array(), index, blockstateOffsets);
    }

    public long getSlabIndex() {
        return this.slabIndex;
    }

}
