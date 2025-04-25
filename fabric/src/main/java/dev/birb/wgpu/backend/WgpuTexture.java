package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.TextureFormat;
import dev.birb.wgpu.rust.WgpuNative;
import lombok.Getter;

import java.util.concurrent.atomic.AtomicBoolean;

//Corresponds one-to-one with a wgpu-mc TextureAndView
public class WgpuTexture extends GpuTexture {

    @Getter
    public long texture;
    public AtomicBoolean alive = new AtomicBoolean(true);

    public WgpuTexture(int usage, String string, TextureFormat textureFormat, int width, int height, int mips) {
        super(usage, string, textureFormat, width, height, mips);
        
        int formatId = switch(textureFormat) {
            case RGBA8 -> 0;
            case RED8 -> 1;
            case RED8I -> 2;
            case DEPTH32 -> 3;
        };
        
        this.texture = WgpuNative.createTexture(formatId, width, height, usage);
    }

    @Override
    public void close() {
        boolean wasAlive = alive.compareAndExchange(true, false);
        if(wasAlive) {
            WgpuNative.dropTexture(this.texture);
        } else {
            throw new IllegalStateException("wgpu texture was already dropped");
        }
    }

    @Override
    public boolean isClosed() {
        return !alive.getAcquire();
    }
}
