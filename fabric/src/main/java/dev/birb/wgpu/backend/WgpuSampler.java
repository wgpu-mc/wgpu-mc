package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.AddressMode;
import com.mojang.blaze3d.textures.FilterMode;
import com.mojang.blaze3d.textures.GpuSampler;
import dev.birb.wm.WM;
import lombok.Getter;

import java.lang.foreign.MemorySegment;
import java.util.OptionalDouble;

public class WgpuSampler extends GpuSampler {

    @Getter
    private final MemorySegment nativeSampler;

    public WgpuSampler(WgpuDevice device) {
        this.nativeSampler = WM.create_sampler(device.getWm());
    }

    @Override
    public AddressMode getAddressModeU() {
        return null;
    }

    @Override
    public AddressMode getAddressModeV() {
        return null;
    }

    @Override
    public FilterMode getMinFilter() {
        return null;
    }

    @Override
    public FilterMode getMagFilter() {
        return null;
    }

    @Override
    public int getMaxAnisotropy() {
        return 0;
    }

    @Override
    public OptionalDouble getMaxLod() {
        return OptionalDouble.empty();
    }

    @Override
    public void close() {

    }
}
