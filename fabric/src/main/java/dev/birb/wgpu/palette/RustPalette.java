package dev.birb.wgpu.palette;

import dev.birb.wgpu.rust.WgpuNative;
import lombok.Getter;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.IndexedIterable;
import org.lwjgl.system.MemoryUtil;

import java.nio.IntBuffer;
import java.util.Objects;

@Getter
public class RustPalette {

    @Getter
    private long slabIndex;
    private final IndexedIterable<?> idList;

    public RustPalette(IndexedIterable<?> idList) {
        this.idList = idList;

    }

    public void readPacket(PacketByteBuf buf) {
        this.slabIndex = WgpuNative.createPalette();
        int index = buf.readerIndex();
        int size = buf.readVarInt();
        
        long[] blockstateOffsets = new long[size];

        for(int i=0;i<size;++i) {
            Object object = this.idList.get(buf.readVarInt());
            RustBlockStateAccessor accessor = (RustBlockStateAccessor) object;
            blockstateOffsets[i] = Objects.requireNonNull(accessor).wgpu_mc$getRustBlockStateIndex();
        }

        WgpuNative.paletteReadPacket(this.slabIndex, buf.array(), index, blockstateOffsets);
    }

}