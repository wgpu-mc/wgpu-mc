package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.TextureFormat;
import dev.birb.wm.WM;

import java.lang.foreign.MemorySegment;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuTexture extends GpuTexture {

    final MemorySegment texture;
    private AtomicBoolean closed = new AtomicBoolean();

    public WgpuTexture(int usage, String string, TextureFormat textureFormat, int width, int height, int depthOrLayers, int mips) {
        super(usage, string, textureFormat, width, height, depthOrLayers, mips);


        int formatId = switch(textureFormat) {
            case RGBA8 -> 0;
            case RED8 -> 1;
            case RED8I -> 2;
            case DEPTH32 -> 3;
        };

        texture = WM.create_texture(formatId, width, height, depthOrLayers, usage);
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
