package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.vertex.VertexFormat;
import org.jetbrains.annotations.Nullable;

import java.util.Collection;
import java.util.function.Supplier;

public class WgpuRenderPass implements RenderPass {
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
    public void bindSampler(String name, @Nullable GpuTexture texture) {

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
    public void drawMultipleIndexed(Collection<RenderObject> objects, @Nullable GpuBuffer buffer, @Nullable VertexFormat.IndexType indexType, Collection<String> validationSkippedUniforms) {

    }

    @Override
    public void draw(int offset, int count) {

    }

    @Override
    public void close() {

    }
}
