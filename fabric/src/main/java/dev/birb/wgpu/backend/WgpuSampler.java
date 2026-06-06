package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.AddressMode;
import com.mojang.blaze3d.textures.FilterMode;
import com.mojang.blaze3d.textures.GpuSampler;

import java.util.OptionalDouble;

public class WgpuSampler extends GpuSampler {

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
