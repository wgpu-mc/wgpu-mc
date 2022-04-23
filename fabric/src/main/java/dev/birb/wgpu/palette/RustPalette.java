package dev.birb.wgpu.palette;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.world.chunk.Palette;

import java.lang.ref.Cleaner;
import java.util.function.Predicate;

public class RustPalette<T> implements Palette<T> {

    private final long rustPalettePointer;
    private final long rustIdList;

    private static final Cleaner cleaner = Cleaner.create();

    public RustPalette(long rustIdList) {
        this(WgpuNative.createPalette(), rustIdList);
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

    }

    @Override
    public void writePacket(PacketByteBuf buf) {

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
}
