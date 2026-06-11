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
    private final WgpuDevice device;

    private final static int DONT_MAP = ~(USAGE_MAP_WRITE | USAGE_MAP_READ);

    public WgpuBuffer(WgpuDevice device, String label, int usage, long size, boolean mapped) {
        size = Mth.roundToward(size, 16);
        super(usage, size);

        this.device = device;

        if((usage & USAGE_MAP_WRITE) != 0) {
            usage |= USAGE_COPY_DST;
        }

        if((usage & USAGE_MAP_READ) != 0) {
            usage |= USAGE_COPY_SRC;
        }

        if((usage & DONT_MAP) != 0) {
            usage &= ~(USAGE_MAP_WRITE | USAGE_MAP_READ);
        }
//        if((usage & USAGE_MAP_WRITE) != 0) usage |= USAGE_COPY_DST;

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
        super(usage, Mth.roundToward(data.capacity(), 16));

        if((usage & USAGE_MAP_WRITE) != 0) {
            usage |= USAGE_COPY_DST;
        }

        if((usage & USAGE_MAP_READ) != 0) {
            usage |= USAGE_COPY_SRC;
        }

        if((usage & DONT_MAP) != 0) {
            usage &= ~(USAGE_MAP_WRITE | USAGE_MAP_READ);
        }

        this.device = device;

        try(Arena arena = Arena.ofConfined()) {
            MemorySegment labelSeg = arena.allocateFrom(label);

            //This function will pad out the data to 16 byte alignment
            nativeBuffer = WM.create_buffer_init(device.getWm(), labelSeg, usage, MemorySegment.ofAddress(MemoryUtil.memAddress0(data)), data.capacity());
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
        if(write) {
            return new GpuBufferSlice.MappedView(new GpuBufferSlice(this, offset, length), data, () -> {
                WM.write_to_buffer(
                        device.getWm(),
                        this.getNativeBuffer(),
                        offset,
                        length,
                        MemorySegment.ofAddress(MemoryUtil.memAddress0(data))
                );
                MemoryUtil.memAlignedFree(data);
            });
        } else {
            return new GpuBufferSlice.MappedView(new GpuBufferSlice(this, offset, length), data, () -> {
                MemoryUtil.memAlignedFree(data);
            });
        }
    }


}
