package dev.birb.wgpu.palette;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import io.netty.buffer.ByteBuf;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.world.chunk.IdListPalette;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PaletteResizeListener;

import java.lang.ref.Cleaner;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.List;
import java.util.function.Predicate;

public class RustPalette<T> implements Palette<T> {

    private long rustPalettePointer;
    private final long rustIdList;

    private static final Cleaner cleaner = Cleaner.create();

    public RustPalette(long rustIdList) {
        this(WgpuNative.createPalette(rustIdList), rustIdList);
    }

    public RustPalette(long rustPalettePointer, long rustIdList) {
        this.rustPalettePointer = rustPalettePointer;
        this.rustIdList = rustIdList;

        cleaner.register(this, () -> WgpuNative.destroyPalette(rustPalettePointer));
    }

    @Override
    public int index(T object) {
        return WgpuNative.paletteIndex(this.rustPalettePointer, object);
    }

    @Override
    public boolean hasAny(Predicate<T> predicate) {
        return WgpuNative.paletteHasAny(this.rustPalettePointer, predicate);
    }

    @Override
    public T get(int id) {
        return (T) WgpuNative.paletteGet(this.rustPalettePointer, id);
    }

    @Override
    public void readPacket(PacketByteBuf buf) {
        assert buf.nioBuffer().order() == ByteOrder.LITTLE_ENDIAN;

        WgpuNative.destroyPalette(this.rustPalettePointer);
        this.rustPalettePointer = WgpuNative.createPalette(this.rustIdList);

        int index = buf.readerIndex();
        buf.readerIndex(index + WgpuNative.paletteReadPacket(this.rustPalettePointer, buf.array(), index));
    }

    @Override
    public void writePacket(PacketByteBuf buf) {
        System.out.println("tried to write packet");
    }

    @Override
    public int getPacketSize() {
        return 0;
    }

    @Override
    public int getSize() {
        return WgpuNative.paletteSize(this.rustPalettePointer);
    }

    @Override
    public Palette<T> copy() {
        return new RustPalette<T>(WgpuNative.copyPalette(this.rustPalettePointer), this.rustIdList);
    }

    public static <A> Palette<A> create(int bits, IndexedIterable<A> idList, PaletteResizeListener<A> listener, List<A> list) {
        return new RustPalette<A>(WgpuNative.uploadIdList((IndexedIterable<Object>) idList));
    }

}
