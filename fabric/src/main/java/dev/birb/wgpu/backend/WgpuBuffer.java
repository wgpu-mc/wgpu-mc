package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import dev.birb.wm.WM;
import lombok.Getter;
import net.minecraft.util.Mth;
import org.jspecify.annotations.NonNull;
import org.lwjgl.system.MemoryUtil;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuBuffer extends GpuBuffer {

    @Getter
    private final MemorySegment nativeBuffer;
    private final AtomicBoolean closed = new AtomicBoolean(false);

    public WgpuBuffer(WgpuDevice device, String label, int usage, long size, boolean mapped) {
        super(usage, size);

        try(Arena arena = Arena.ofConfined()) {
            MemorySegment labelSeg = arena.allocateFrom(label);

            if(!mapped) {
                nativeBuffer = WM.create_buffer(
                        device.getWm(),
                        labelSeg,
                        usage,
                        Mth.roundToward(size, 16)
                );
            } else {
                nativeBuffer = WM.allocate_gpu_buffer_mapped(
                        device.getWm(),
                        Mth.roundToward(size, 16),
                        usage
                );
            }
        }
    }

    public WgpuBuffer(WgpuDevice device, String label, int usage, ByteBuffer data) {
        super(usage, data.capacity());

        try(Arena arena = Arena.ofConfined()) {
            MemorySegment labelSeg = arena.allocateFrom(label);

            nativeBuffer = WM.create_buffer_init(device.getWm(), labelSeg, usage, MemorySegment.ofBuffer(data), Mth.roundToward(data.capacity(), 16));
        }
    }

    @Override
    public boolean isClosed() {
        return closed.get();
    }

    @Override
    public void close() {
        if(!closed.compareAndExchange(false, true)) WM.drop_buffer(nativeBuffer);
    }

    @Override
    public GpuBufferSlice.@NonNull MappedView map(long offset, long length, boolean read, boolean write) {
        ByteBuffer data = MemoryUtil.memAlignedAlloc(16, (int) length);
        return new GpuBufferSlice.MappedView(new GpuBufferSlice(this, offset, length), data, () -> {
            MemoryUtil.memFree(data);
        });
    }


}
