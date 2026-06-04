package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.systems.RenderPassBackend;
import com.mojang.blaze3d.textures.GpuSampler;
import com.mojang.blaze3d.textures.GpuTextureView;
import com.mojang.blaze3d.vertex.VertexFormat;

import java.util.Collection;
import java.util.function.Supplier;

public class WgpuRenderPass implements RenderPassBackend {
    @Override
    public void pushDebugGroup(Supplier<String> supplier) {

    }

    @Override
    public void popDebugGroup() {

    }

    @Override
    public void setPipeline(RenderPipeline pipeline) {

    }

    @Override
    public void bindTexture(String name, @org.jspecify.annotations.Nullable GpuTextureView textureView, @org.jspecify.annotations.Nullable GpuSampler sampler) {

    }


    @Override
    public void setUniform(String name, GpuBuffer buffer) {

    }

    @Override
    public void setUniform(String name, GpuBufferSlice slice) {

    }

    @Override
    public void enableScissor(int x, int y, int width, int height) {

    }

    @Override
    public void disableScissor() {

    }

    @Override
    public void setVertexBuffer(int index, GpuBuffer buffer) {

    }

    @Override
    public void setIndexBuffer(GpuBuffer indexBuffer, VertexFormat.IndexType indexType) {

    }

    @Override
    public void drawIndexed(int offset, int count, int primcount, int i) {

    }

    @Override
    public <T> void drawMultipleIndexed(Collection<RenderPass.Draw<T>> draws, @org.jspecify.annotations.Nullable GpuBuffer defaultIndexBuffer, VertexFormat.@org.jspecify.annotations.Nullable IndexType defaultIndexType, Collection<String> dynamicUniforms, T uniformArgument) {

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
