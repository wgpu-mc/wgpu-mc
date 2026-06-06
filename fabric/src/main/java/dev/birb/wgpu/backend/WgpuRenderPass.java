package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.systems.RenderPassBackend;
import com.mojang.blaze3d.textures.GpuSampler;
import com.mojang.blaze3d.textures.GpuTextureView;
import com.mojang.blaze3d.vertex.VertexFormat;
import dev.birb.wm.WM;
import org.jspecify.annotations.NonNull;
import org.jspecify.annotations.Nullable;

import java.lang.foreign.MemorySegment;
import java.util.Collection;
import java.util.OptionalDouble;
import java.util.OptionalInt;
import java.util.function.Supplier;

public class WgpuRenderPass implements RenderPassBackend {

    private final MemorySegment nativePass;
    private final WgpuDevice device;

    public WgpuRenderPass(MemorySegment nativeCommandEncoder, String s, @NonNull WgpuTextureView colorTexture, @NonNull OptionalInt clearColor, WgpuDevice device) {
        this.device = device;
        nativePass = WM.create_render_pass(nativeCommandEncoder, colorTexture.nativeView, clearColor.isPresent(), clearColor.isPresent() ? clearColor.getAsInt() : 0, MemorySegment.NULL, false, 0.0);
    }

    public WgpuRenderPass(MemorySegment nativeCommandEncoder, String s, @NonNull WgpuTextureView colorTexture, @NonNull OptionalInt clearColor, @Nullable WgpuTextureView depthTexture, @NonNull OptionalDouble clearDepth, WgpuDevice device) {
        this.device = device;
        nativePass = WM.create_render_pass(
                nativeCommandEncoder,
                colorTexture.nativeView,
                clearColor.isPresent(),
                clearColor.isPresent() ? clearColor.getAsInt() : 0,
                depthTexture != null ? depthTexture.nativeView : MemorySegment.NULL,
                clearDepth.isPresent(),
                clearDepth.isPresent() ? clearDepth.getAsDouble() : 0.0
        );
    }


    @Override
    public void pushDebugGroup(@NonNull Supplier<String> supplier) {

    }

    @Override
    public void popDebugGroup() {

    }

    @Override
    public void setPipeline(@NonNull RenderPipeline pipeline) {
        WgpuCompiledRenderPipeline wgpuPipeline = WgpuCompiledRenderPipeline.wgpuRenderPipelines.computeIfAbsent(pipeline, p -> new WgpuCompiledRenderPipeline(p, this.device.getDefaultShaderSource()));
        WM.bind_render_pipeline_to_pass(nativePass, wgpuPipeline.getNativePipeline());
    }

    @Override
    public void bindTexture(@NonNull String name, @org.jspecify.annotations.Nullable GpuTextureView textureView, @org.jspecify.annotations.Nullable GpuSampler sampler) {
//        WM.bind_texture_to_render_pass(nativePass, );
    }


    @Override
    public void setUniform(@NonNull String name, @NonNull GpuBuffer buffer) {

    }

    @Override
    public void setUniform(@NonNull String name, @NonNull GpuBufferSlice slice) {

    }

    @Override
    public void enableScissor(int x, int y, int width, int height) {

    }

    @Override
    public void disableScissor() {

    }

    @Override
    public void setVertexBuffer(int index, @NonNull GpuBuffer buffer) {
        
    }

    @Override
    public void setIndexBuffer(@NonNull GpuBuffer indexBuffer, VertexFormat.@NonNull IndexType indexType) {
        
    }

    @Override
    public void drawIndexed(int offset, int count, int primcount, int i) {

    }

    @Override
    public <T> void drawMultipleIndexed(@NonNull Collection<RenderPass.Draw<T>> draws, @org.jspecify.annotations.Nullable GpuBuffer defaultIndexBuffer, VertexFormat.@org.jspecify.annotations.Nullable IndexType defaultIndexType, @NonNull Collection<String> dynamicUniforms, @NonNull T uniformArgument) {

    }

    @Override
    public void draw(int offset, int count) {

    }

    @Override
    public void close() {

    }

    @Override
    public boolean isClosed() {
        return false;
    }
}
