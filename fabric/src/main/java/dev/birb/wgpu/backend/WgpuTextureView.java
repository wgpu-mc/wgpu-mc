package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.GpuTextureView;
import dev.birb.wm.WM;
import lombok.Getter;

import java.lang.foreign.MemorySegment;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuTextureView extends GpuTextureView {

    @Getter
    private final MemorySegment nativeView;
    private final AtomicBoolean closed = new AtomicBoolean();

    protected WgpuTextureView(WgpuTexture texture, int baseMipLevel, int mipLevels) {
        super(texture, baseMipLevel, mipLevels);

        nativeView = WM.create_texture_view(texture.texture);
    }

    @Override
    public void close() {
        if(!closed.compareAndExchange(false, true)) WM.drop_texture_view(nativeView);
    }

    @Override
    public boolean isClosed() {
        return closed.get();
    }
}
