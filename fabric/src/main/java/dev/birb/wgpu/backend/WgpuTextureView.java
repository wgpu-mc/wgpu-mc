package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.GpuTextureView;
import dev.birb.wm.WM;
import lombok.Getter;

import java.lang.foreign.MemorySegment;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuTextureView extends GpuTextureView implements NativeResource {

    private final MemorySegment nativeView;
    private final AtomicBoolean closed = new AtomicBoolean();
    @Getter
    private final WgpuDevice device;

    protected WgpuTextureView(WgpuDevice device, WgpuTexture texture, int baseMipLevel, int mipLevels) {
        super(texture, baseMipLevel, mipLevels);
        this.device = device;

        var isCube = (texture.usage() & GpuTexture.USAGE_CUBEMAP_COMPATIBLE) != 0;

        //0 = None for the wgpu texture view descriptor which just re-interprets it as 1
        var arraySize = isCube ? 6 : 0;

        nativeView = WM.create_texture_view(this.device.getWm(), texture.texture, texture.usage(), baseMipLevel, mipLevels, arraySize);
    }

    @Override
    public void close() {
        if(!closed.compareAndExchange(false, true)) WM.drop_texture_view(nativeView);
    }

    @Override
    public MemorySegment getNativeUnsafe() {
        return this.nativeView;
    }

    @Override
    public boolean isClosed() {
        return closed.get();
    }
}
