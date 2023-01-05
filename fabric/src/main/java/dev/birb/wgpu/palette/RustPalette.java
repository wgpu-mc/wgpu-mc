package dev.birb.wgpu.palette;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PaletteResizeListener;

import java.nio.ByteOrder;
import java.util.List;
import java.util.function.Predicate;

public class RustPalette {

    private final long slabIndex;

//    public static final Cleaner CLEANER = Cleaner.create();

    public RustPalette(long rustIdList) {
        this.slabIndex = WgpuNative.createPalette(rustIdList);
    }

    public void readPacket(PacketByteBuf buf) {
//        assert buf.nioBuffer().order() == ByteOrder.LITTLE_ENDIAN;
//
//        WgpuNative.clearPalette(this.slabIndex);
//
        int index = buf.readerIndex();
//
        int size = buf.readVarInt();
//
        int[] blockstateOffsets = new int[size];
//
//        for(int i=0;i<size;i++) {
//            T object = this.idList.get(buf.readVarInt());
//            RustBlockStateAccessor accessor = (RustBlockStateAccessor) object;
//            blockstateOffsets[i] = accessor.getRustBlockStateIndex();
//        }
//
//        int javaRead = buf.readerIndex();


        WgpuNative.paletteReadPacket(this.slabIndex, buf.array(), index, blockstateOffsets);
//        assert javaRead - index == wmRead;
    }

    public void writePacket(PacketByteBuf buf) {
        int size = this.getSize();
        buf.writeVarInt(size);
        for (int i = 0; i < size; ++i) {
            buf.writeVarInt(this.idList.getRawId(this.get(i)));
        }
    }

    public int getPacketSize() {
        int i = PacketByteBuf.getVarIntLength(this.getSize());
        for (int j = 0; j < this.getSize(); ++j) {
            i += PacketByteBuf.getVarIntLength(this.idList.getRawId(this.get(i)));
        }
        return i;
    }

    public int getSize() {
        return WgpuNative.paletteSize(this.slabIndex);
    }

    public RustPalette copy() {
        return new RustPalette(WgpuNative.copyPalette(this.slabIndex), this.rustIdList);
    }

    public static <A> Palette<A> create(int bits, IndexedIterable<A> idList, PaletteResizeListener<A> listener, List<A> list) {
        return new RustPalette<A>(WgpuNative.uploadIdList((IndexedIterable<Object>) idList), idList, listener, bits);
    }

    public long getSlabIndex() {
        return this.slabIndex;
    }

}
