package dev.birb.wgpu.backend;

import com.mojang.blaze3d.GpuFormat;
import com.mojang.blaze3d.textures.GpuTexture;
import dev.birb.wgpu.helper.GpuFormatHelper;
import dev.birb.wm.WM;

import java.lang.foreign.MemorySegment;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuTexture extends GpuTexture {

    final MemorySegment texture;
    private AtomicBoolean closed = new AtomicBoolean();

    public WgpuTexture(int usage, String string, GpuFormat gpuFormat, int width, int height, int depthOrLayers, int mips) {
        super(usage, string, gpuFormat, width, height, depthOrLayers, mips);

        texture = WM.create_texture(GpuFormatHelper.gpuFormatToRustEnum(gpuFormat), width, height, depthOrLayers, usage);
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
