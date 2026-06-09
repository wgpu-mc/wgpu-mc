package dev.birb.wgpu.backend;

import com.mojang.blaze3d.textures.AddressMode;
import com.mojang.blaze3d.textures.FilterMode;
import com.mojang.blaze3d.textures.GpuSampler;
import dev.birb.wm.WM;
import lombok.Getter;
import org.jspecify.annotations.NonNull;

import java.lang.foreign.MemorySegment;
import java.util.OptionalDouble;

public class WgpuSampler extends GpuSampler {

    @Getter
    private final MemorySegment nativeSampler;

    public WgpuSampler(WgpuDevice device, @NonNull AddressMode addressModeU, @NonNull AddressMode addressModeV, @NonNull FilterMode minFilter, @NonNull FilterMode magFilter, int maxAnisotropy, @NonNull OptionalDouble maxLod) {
        this.nativeSampler = WM.create_sampler(
                device.getWm(),
                switch(addressModeU) {
                    case REPEAT -> 0;
                    case CLAMP_TO_EDGE -> 1;
                },
                switch(addressModeV) {
                    case REPEAT -> 0;
                    case CLAMP_TO_EDGE -> 1;
                },
                switch(minFilter) {
                    case NEAREST -> 0;
                    case LINEAR -> 1;
                },
                switch(magFilter) {
                    case NEAREST -> 0;
                    case LINEAR -> 1;
                },
                maxAnisotropy,
                //TODO
                0.0
        );
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
