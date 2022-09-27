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

public class RustPalette<T> implements Palette<T> {

    private final long rustPalettePointer;
    private final long rustIdList;
    private final IndexedIterable<T> idList;
    private final PaletteResizeListener<T> listener;
    private final int bits;

    public static final Cleaner CLEANER = Cleaner.create();

    public RustPalette(long rustIdList, IndexedIterable<T> idList, PaletteResizeListener<T> listener, int bits) {
        this(WgpuNative.createPalette(rustIdList), rustIdList, idList, listener, bits);
    }

    public RustPalette(long rustPalettePointer, long rustIdList, IndexedIterable<T> idList, PaletteResizeListener<T> listener, int bits) {
        this.rustPalettePointer = rustPalettePointer;
        this.rustIdList = rustIdList;
        this.idList = idList;
        this.listener = listener;
        this.bits = bits;

        CLEANER.register(this, () -> WgpuNative.destroyPalette(rustPalettePointer));
    }

    @Override
    public int index(T object) {
        RustBlockStateAccessor accessor = (RustBlockStateAccessor) object;

        int index = WgpuNative.paletteIndex(this.rustPalettePointer, object, accessor.getRustBlockStateIndex());
        if(index >= (1 << this.bits)) {
            System.out.println("Resizing palette to " + (this.bits+1));
            index = this.listener.onResize(this.bits + 1, object);
        }
        return index;
    }

    @Override
    public boolean hasAny(Predicate<T> predicate) {
        for(int i=0;i<this.getSize();i++) {
            T t = this.get(i);
            if(predicate.test(t)) return true;
        }

        return false;
    }

    @Override
    public T get(int id) {
        T t = (T) WgpuNative.paletteGet(this.rustPalettePointer, id);
        return t;
    }

    @Override
    public void readPacket(PacketByteBuf buf) {
        assert buf.nioBuffer().order() == ByteOrder.LITTLE_ENDIAN;

        WgpuNative.clearPalette(this.rustPalettePointer);

        int index = buf.readerIndex();

        int size = buf.readVarInt();

        int[] blockstateOffsets = new int[size];

        for(int i=0;i<size;i++) {
            T object = this.idList.get(buf.readVarInt());
            RustBlockStateAccessor accessor = (RustBlockStateAccessor) object;
            blockstateOffsets[i] = accessor.getRustBlockStateIndex();
        }

        int javaRead = buf.readerIndex();

        int wmRead = WgpuNative.paletteReadPacket(this.rustPalettePointer, buf.array(), index, blockstateOffsets);
        assert javaRead - index == wmRead;
    }

    @Override
    public void writePacket(PacketByteBuf buf) {
        int size = this.getSize();
        buf.writeVarInt(size);
        for (int i = 0; i < size; ++i) {
            buf.writeVarInt(this.idList.getRawId(this.get(i)));
        }
    }

    @Override
    public int getPacketSize() {
        int i = PacketByteBuf.getVarIntLength(this.getSize());
        for (int j = 0; j < this.getSize(); ++j) {
            i += PacketByteBuf.getVarIntLength(this.idList.getRawId(this.get(i)));
        }
        return i;
    }

    @Override
    public int getSize() {
        return WgpuNative.paletteSize(this.rustPalettePointer);
    }

    @Override
    public Palette<T> copy() {
        return new RustPalette<T>(WgpuNative.copyPalette(this.rustPalettePointer), this.rustIdList, this.idList, this.listener, this.bits);
    }

    public static <A> Palette<A> create(int bits, IndexedIterable<A> idList, PaletteResizeListener<A> listener, List<A> list) {
        return new RustPalette<A>(WgpuNative.uploadIdList((IndexedIterable<Object>) idList), idList, listener, bits);
    }

    public long getRustPointer() {
        return this.rustPalettePointer;
    }

}
