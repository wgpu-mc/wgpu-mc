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
    private final long slabIndex;
    private final IndexedIterable<?> idList;

    public RustPalette(IndexedIterable<?> idList, long rustIdList) {
        this.idList = idList;
        this.slabIndex = WgpuNative.createPalette(rustIdList);
    }

    public void readPacket(PacketByteBuf buf) {
        int index = buf.readerIndex();
        int size = buf.readVarInt();
        
        IntBuffer blockstateOffsets = MemoryUtil.memAllocInt(size);

        for(int i=0;i<size;++i) {
            Object object = this.idList.get(buf.readVarInt());
            RustBlockStateAccessor accessor = (RustBlockStateAccessor) object;
            blockstateOffsets.put(i, Objects.requireNonNull(accessor).wgpu_mc$getRustBlockStateIndex());
        }

        WgpuNative.paletteReadPacket(this.slabIndex, buf.array(), index, MemoryUtil.memAddress0(blockstateOffsets), size);
    }

}