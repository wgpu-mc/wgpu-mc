package dev.birb.wgpu.backend;

import com.mojang.blaze3d.GpuFormat;
import com.mojang.blaze3d.textures.GpuTexture;
import dev.birb.wgpu.helper.GpuFormatHelper;
import dev.birb.wm.WM;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuTexture extends GpuTexture {

    final MemorySegment texture;
    private AtomicBoolean closed = new AtomicBoolean();

    public WgpuTexture(WgpuDevice device, int usage, String name, GpuFormat gpuFormat, int width, int height, int depthOrLayers, int mips) {
        super(usage, name, gpuFormat, width, height, depthOrLayers, mips);
        
        try(Arena arena = Arena.ofConfined()) {
            texture = WM.create_texture(device.getWm(), GpuFormatHelper.gpuFormatToRustEnum(gpuFormat), width, height, depthOrLayers, usage, arena.allocateFrom(name));
        }
    }

    @Override
    public void close() {
        if(!closed.compareAndExchange(false, true)) WM.drop_texture(texture);
    }

    @Override
    public boolean isClosed() {
        return closed.get();
    }
}
