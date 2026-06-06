package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import dev.birb.wm.WM;
import lombok.Getter;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryUtil;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuBuffer extends GpuBuffer {

    @Getter
    private final MemorySegment nativeBuffer;
    private final AtomicBoolean closed = new AtomicBoolean(false);

    public WgpuBuffer(String label, int usage, long size) {
        super(usage, size);

        try(Arena arena = Arena.ofConfined()) {
            MemorySegment labelSeg = arena.allocateFrom(label);

            nativeBuffer = WM.create_buffer(labelSeg, usage, size);
        }
    }

    public WgpuBuffer(String label, int usage, ByteBuffer data) {
        super(usage, data.capacity());

        try(Arena arena = Arena.ofConfined()) {
            MemorySegment labelSeg = arena.allocateFrom(label);

            nativeBuffer = WM.create_buffer_init(labelSeg, usage, MemorySegment.ofBuffer(data), data.capacity());
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

    public static class WgpuMappedView implements MappedView {

        private final ByteBuffer data;
        private final WgpuBuffer buffer;

        public WgpuMappedView(long size, WgpuBuffer buffer) {
            this.data = MemoryUtil.memAlloc((int) size);
            this.buffer = buffer;
        }

        @Override
        public @NotNull ByteBuffer data() {
            return data;
        }

        @Override
        public void close() {
//            WM.write_mapped_buffer(buffer.nativeBuffer, MemorySegment.ofBuffer(data), data.capacity());
        }
    }

}
