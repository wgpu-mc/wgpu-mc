package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.systems.TransientMemory;
import dev.birb.wm.WM;
import net.minecraft.util.Mth;
import org.jspecify.annotations.NonNull;

import java.io.Closeable;
import java.io.IOException;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.List;

public class WgpuTransientMemory implements TransientMemory, Closeable {

    private final WgpuDevice device;
    private final Arena arena = Arena.ofConfined();

    public WgpuTransientMemory(WgpuDevice device, WgpuCommandEncoder encoder) {
        this.device = device;
    }

    @Override
    public @NonNull ByteBuffer allocateCpu(long size, long alignment, long minimumAllocation, long elementSize) {
        return arena.allocate(Mth.roundToward(size, alignment), alignment).asByteBuffer();
    }

    @Override
    public GpuBufferSlice.@NonNull MappedView allocateStaging(long size, long alignment, @GpuBuffer.Usage int usage, long minimumAllocation, long elementSize) {
        return this.allocateGpuMapped(size, alignment, usage, minimumAllocation, elementSize);
    }

    @Override
    public @NonNull GpuBufferSlice allocateGpu(long size, long alignment, @GpuBuffer.Usage int usage, long minimumAllocation, long elementSize) {
        int newSize = Mth.roundToward((int) size, (int) Math.max(alignment, minimumAllocation));

        var buffer = new WgpuBuffer(device, "", usage, newSize, false);

        return new GpuBufferSlice(buffer, 0, size);
    }

    @Override
    public GpuBufferSlice.@NonNull MappedView allocateGpuMapped(long size, long alignment, @GpuBuffer.Usage int usage, long minimumAllocation, long elementSize) {
        var newSize = Mth.roundToward(size, alignment);
        var buffer = new WgpuBuffer(device, "", usage, newSize, false);

        ByteBuffer cpuBuffer = arena.allocate(newSize, alignment).asByteBuffer();

        return new GpuBufferSlice.MappedView(
                new GpuBufferSlice(buffer, 0, newSize),
                cpuBuffer,
                () -> {
                    WM.write_buffer_with(device.getWm(), buffer.getNativeBuffer(), MemorySegment.ofBuffer(cpuBuffer), newSize);
                }
        );
    }

    @Override
    public @NonNull GpuBufferSlice uploadStaging(@NonNull List<ByteBuffer> data, long alignment, @GpuBuffer.Usage int usage, long minimumAllocation, long elementSize) {
        var arena = Arena.ofAuto();
        var totalSize = 0;

        for(var buffer : data) {
            totalSize += Mth.roundToward(buffer.limit(), (int) alignment);
        }

        var bigBuffer = arena.allocate(totalSize, alignment);

        long offset = 0;
        for(var buffer : data) {
            bigBuffer.asSlice(offset).copyFrom(MemorySegment.ofBuffer(buffer));
            offset += buffer.limit() + Mth.roundToward(buffer.limit(), (int) alignment);
        }

        return new GpuBufferSlice(
                new WgpuBuffer(device, "", usage, bigBuffer.asByteBuffer()),
                0,
                totalSize
        );
    }

    @Override
    public @NonNull GpuBufferSlice uploadGpu(@NonNull List<ByteBuffer> data, long alignment, @GpuBuffer.Usage int usage, long minimumAllocation, long elementSize) {
        return this.uploadStaging(data, alignment, usage, minimumAllocation, elementSize);
    }

    @Override
    public @NonNull List<GpuBufferSlice> multiUploadStaging(@NonNull List<ByteBuffer> data, long alignment, @GpuBuffer.Usage int usage) {
        var out = new ArrayList<GpuBufferSlice>();

        for(var buf : data) {
            out.add(this.uploadStaging(buf, alignment, usage));
        }

        return out;
    }

    @Override
    public @NonNull List<GpuBufferSlice> multiUploadGpu(@NonNull List<ByteBuffer> data, long alignment, @GpuBuffer.Usage int usage) {
        var out = new ArrayList<GpuBufferSlice>();

        for(var buf : data) {
            out.add(this.uploadGpu(buf, alignment, usage));
        }

        return out;
    }


    @Override
    public void close() throws IOException {
        this.arena.close();
    }
}
