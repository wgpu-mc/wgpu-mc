package dev.birb.wgpu.backend;

import com.mojang.blaze3d.GpuFormat;
import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.systems.*;
import com.mojang.blaze3d.textures.*;
import dev.birb.wgpu.rust.WgpuNative;
import lombok.Getter;
import org.jetbrains.annotations.Nullable;
import org.jspecify.annotations.NonNull;

import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.List;
import java.util.OptionalDouble;
import java.util.OptionalLong;
import java.util.Set;
import java.util.function.Supplier;

public class WgpuDevice implements GpuDeviceBackend {

    @Getter
    private final ShaderSource defaultShaderSource;

    @Getter
    private final MemorySegment wm;

    // private final BiFunction<Identifier, ShaderType, String> shaderSourceGetter;

    public WgpuDevice(ShaderSource defaultShaderSource) {
        this.defaultShaderSource = defaultShaderSource;

        this.wm = MemorySegment.ofAddress(WgpuNative.create_device());
    }

    @Override
    public @NonNull GpuSurfaceBackend createSurface(long windowHandle) {
        return new WgpuSurface(this, windowHandle);
    }

    @Override
    public @NonNull CommandEncoderBackend createCommandEncoder() {
        return new WgpuCommandEncoder(this);
    }

    @Override
    public @NonNull GpuSampler createSampler(@NonNull AddressMode addressModeU, @NonNull AddressMode addressModeV,
            @NonNull FilterMode minFilter, @NonNull FilterMode magFilter, int maxAnisotropy,
            @NonNull OptionalDouble maxLod) {
        return new WgpuSampler();
    }

    @Override
    public @NonNull GpuTexture createTexture(@org.jspecify.annotations.Nullable Supplier<String> label, @GpuTexture.Usage int usage, @NonNull GpuFormat format, int width, int height, int depthOrLayers, int mipLevels) {
        return new WgpuTexture(
                this,
                usage,
                label != null ? label.get() : "<wm/unnamed mc texture>",
                format,
                width,
                height,
                depthOrLayers,
                mipLevels
        );
    }

    @Override
    public @NonNull GpuTexture createTexture(@org.jspecify.annotations.Nullable String label, @GpuTexture.Usage int usage, @NonNull GpuFormat format, int width, int height, int depthOrLayers, int mipLevels) {
        return new WgpuTexture(
                this,
                usage,
                label,
                format,
                width,
                height,
                depthOrLayers,
                mipLevels
        );
    }

    @Override
    public @NonNull GpuTextureView createTextureView(@NonNull GpuTexture texture) {
        return new WgpuTextureView(this, (WgpuTexture) texture, 1, 1);
    }

    @Override
    public @NonNull GpuTextureView createTextureView(@NonNull GpuTexture texture, int baseMipLevel, int mipLevels) {
        return new WgpuTextureView(this, (WgpuTexture) texture, baseMipLevel, mipLevels);
    }

    @Override
    public @NonNull GpuBuffer createBuffer(@org.jspecify.annotations.Nullable Supplier<String> label,
            @GpuBuffer.Usage int usage, long size) {
        return new WgpuBuffer(this, label != null ? label.get() : "<wm/unnamed mc buffer>", usage, size);
    }

    @Override
    public @NonNull GpuBuffer createBuffer(@Nullable Supplier<String> labelGetter, int usage,
            @NonNull ByteBuffer data) {
        return new WgpuBuffer(this, labelGetter != null ? labelGetter.get() : "<wm/unnamed mc buffer>", usage, data);
    }

    @Override
    public @NonNull List<String> getLastDebugMessages() {
        return List.of();
    }

    @Override
    public boolean isDebuggingEnabled() {
        return false;
    }

    @Override
    public @NonNull CompiledRenderPipeline precompilePipeline(@NonNull RenderPipeline pipeline,
            @org.jspecify.annotations.Nullable ShaderSource shaderSource) {
        var source = shaderSource != null ? shaderSource : this.defaultShaderSource;

        return WgpuCompiledRenderPipeline.wgpuRenderPipelines.computeIfAbsent(pipeline,
                p -> new WgpuCompiledRenderPipeline(this, p, source));
    }

    @Override
    public void clearPipelineCache() {
        WgpuCompiledRenderPipeline.wgpuRenderPipelines.clear();
        WgpuCompiledRenderPipeline.shaderSourceCache.clear();
    }

    @Override
    public void close() {

    }

    @Override
    public @NonNull GpuQueryPool createTimestampQueryPool(int size) {
        return new GpuQueryPool() {
            @Override
            public int size() {
                return 0;
            }

            @Override
            public @NonNull OptionalLong getValue(int index) {
                return OptionalLong.empty();
            }

            @Override
            public OptionalLong @NonNull [] getValues(int index, int count) {
                return new OptionalLong[0];
            }

            @Override
            public void close() {

            }
        };
    }

    @Override
    public long getTimestampNow() {
        return 0;
    }

    @Override
    public @NonNull DeviceInfo getDeviceInfo() {
        return new DeviceInfo(
                "wgpu",
                "wgpu",
                "-",
                false,
                "wgpu-mc",
                1.0f,
                new DeviceLimits(1, 1, 4096, 100000000, 1, 4),
                new DeviceFeatures(true, false, false, false, false, false, false),
                Set.of(),
                new HintsAndWorkarounds(false, true),
                DeviceType.DISCRETE
        );
    }


}
