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
        int index = buf.readerIndex();
        int size = buf.readVarInt();

        int[] blockstateOffsets = new int[size];

        WgpuNative.paletteReadPacket(this.slabIndex, buf.array(), index, blockstateOffsets);
    }

    public long getSlabIndex() {
        return this.slabIndex;
    }

}
