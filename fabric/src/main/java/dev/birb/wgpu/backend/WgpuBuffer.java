package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import dev.birb.wgpu.rust.WgpuNative;
import lombok.Getter;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuBuffer extends GpuBuffer {

    private long buffer;
    private long mapShadow;
    @Getter
    private ByteBuffer map;

    public AtomicBoolean alive = new AtomicBoolean(true);

    public WgpuBuffer(String label, int usage, int size) {
        super(usage, size);

//        if((usage & GpuBuffer.USAGE_MAP_READ) != 0 || (usage & GpuBuffer.USAGE_MAP_WRITE) != 0) {
//            this.mapShadow = WgpuNative.createBuffer(label, usage, size);
//        }

        this.map = MemoryUtil.memAlloc(size);
        this.buffer = WgpuNative.createBuffer(label, usage & ~(GpuBuffer.USAGE_MAP_WRITE | GpuBuffer.USAGE_MAP_READ), size);
    }

    public WgpuBuffer(String label, int usage, ByteBuffer data) {
        super(usage, data.capacity());

        this.map = MemoryUtil.memAlloc(size);
        MemoryUtil.memCopy(data, this.map);
        this.buffer = WgpuNative.createBufferInit(label, usage & ~(GpuBuffer.USAGE_MAP_WRITE | GpuBuffer.USAGE_MAP_READ), data);
    }

    @Override
    public boolean isClosed() {
        return !alive.getAcquire();
    }

    @Override
    public void close() {
        boolean wasAlive = alive.compareAndExchange(true, false);
        if(wasAlive) {
            WgpuNative.dropBuffer(this.buffer);
        } else {
            throw new IllegalStateException("wgpu buffer was already dropped");
        }
    }

    public static class WgpuMappedView implements MappedView {

        private ByteBuffer buffer;

        public WgpuMappedView(ByteBuffer buffer) {
            this.buffer = buffer;
        }

        @Override
        public ByteBuffer data() {
            return this.buffer;
        }

        @Override
        public void close() {

        }

    }

}
