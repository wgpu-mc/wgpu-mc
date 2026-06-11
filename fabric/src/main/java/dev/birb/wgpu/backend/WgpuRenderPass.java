package dev.birb.wgpu.backend;

import com.mojang.blaze3d.IndexType;
import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.systems.GpuQueryPool;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.systems.RenderPassBackend;
import com.mojang.blaze3d.systems.RenderPassDescriptor;
import com.mojang.blaze3d.textures.GpuSampler;
import com.mojang.blaze3d.textures.GpuTextureView;
import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wm.*;
import lombok.Getter;
import net.minecraft.util.Mth;
import org.jspecify.annotations.NonNull;
import org.jspecify.annotations.Nullable;
import org.lwjgl.PointerBuffer;

import java.io.Closeable;
import java.io.IOException;
import java.lang.foreign.*;
import java.nio.IntBuffer;
import java.util.Collection;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.BiConsumer;
import java.util.function.Supplier;

public class WgpuRenderPass implements RenderPassBackend, Closeable {

    @Getter
    private final MemorySegment nativePass;
    private final WgpuDevice device;
    private static final MemoryLayout vec4fLayout = MemoryLayout.sequenceLayout(
            4, ValueLayout.JAVA_FLOAT
    );

    private boolean rebuildBindGroups = true;
    private MemorySegment bindGroupCache;

    @Nullable
    private final MemorySegment bindingBuilder = WM.create_binding_builder();

    @Nullable
    private MemorySegment activePipeline;
    private final AtomicBoolean closed = new AtomicBoolean();
    private final boolean wantsDepth;

    public WgpuRenderPass(WgpuDevice device, WgpuCommandEncoder encoder, RenderPassDescriptor descriptor) {
        this.device = device;

        this.wantsDepth = descriptor.depthAttachment != null;

        try(Arena arena = Arena.ofConfined()) {
            var rawRenderPass = BlazeRenderPassDescriptor.allocate(arena);

            var attachmentsAllocation = BlazeAttachmentDescriptor__________f32__________4.allocateArray(descriptor.colorAttachments().size(), arena);

            for(int i=0;i<descriptor.colorAttachments().size();i++) {
                var attachment = descriptor.colorAttachments().get(i);

                var attachmentSeg = BlazeAttachmentDescriptor__________f32__________4.asSlice(attachmentsAllocation, i);

                var clearValRaw = MemorySegment.NULL;

                if (attachment != null && attachment.clearValue().isPresent()) {
                    var clearVal = attachment.clearValue().get();
                    clearValRaw = arena.allocate(vec4fLayout);
                    clearValRaw.set(ValueLayout.JAVA_FLOAT, 0, clearVal.get(0));
                    clearValRaw.set(ValueLayout.JAVA_FLOAT, 1, clearVal.get(1));
                    clearValRaw.set(ValueLayout.JAVA_FLOAT, 2, clearVal.get(2));
                    clearValRaw.set(ValueLayout.JAVA_FLOAT, 3, clearVal.get(3));
                }

                var view = (WgpuTextureView) attachment.textureView();

                BlazeAttachmentDescriptor__________f32__________4.texture_view(attachmentSeg, view.getNative());
                BlazeAttachmentDescriptor__________f32__________4.clear_value(attachmentSeg, clearValRaw);
            }

            MemorySegment depthAttachment = MemorySegment.NULL;

            if(descriptor.depthAttachment != null) {
                depthAttachment = BlazeAttachmentDescriptor_f64.allocate(arena);
                var depthView = ((WgpuTextureView) descriptor.depthAttachment().textureView()).getNative();

                MemorySegment clearValue = MemorySegment.NULL;

                if (descriptor.depthAttachment != null && descriptor.depthAttachment.clearValue().isPresent()) {
                    clearValue = arena.allocate(ValueLayout.JAVA_DOUBLE);
                    clearValue.set(ValueLayout.JAVA_DOUBLE, 0, descriptor.depthAttachment().clearValue().getAsDouble());
                }

                BlazeAttachmentDescriptor_f64.texture_view(depthAttachment, depthView);
                BlazeAttachmentDescriptor_f64.clear_value(depthAttachment, clearValue);
            }

            var colorAttachmentsRawArray = RawArray______BlazeAttachmentDescriptor__________f32__________4.allocate(arena);
            RawArray______BlazeAttachmentDescriptor__________f32__________4.size(colorAttachmentsRawArray, descriptor.colorAttachments().size());
            RawArray______BlazeAttachmentDescriptor__________f32__________4.contents(colorAttachmentsRawArray, attachmentsAllocation);

            BlazeRenderPassDescriptor.attachments(rawRenderPass, colorAttachmentsRawArray);
            BlazeRenderPassDescriptor.depth_attachment(rawRenderPass, depthAttachment);

            this.nativePass = WM.create_render_pass(
                    encoder.getNativeCommandEncoder(),
                    rawRenderPass
            );
        }
    }

    @Override
    public void pushDebugGroup(@NonNull Supplier<String> supplier) {

    }

    @Override
    public void popDebugGroup() {

    }

    public MemorySegment buildBindGroups(MemorySegment pipeline) {
        if(this.rebuildBindGroups || this.bindGroupCache == null) {
            if(this.bindGroupCache != null) {
                var a = this.bindGroupCache;
                this.bindGroupCache = null;
                new Thread(() -> WM.drop_bind_groups(a)).start();
            }
            this.bindGroupCache = WM.finalize_binding_builder(device.getWm(), bindingBuilder, pipeline);
            this.rebuildBindGroups = false;
        }

        return this.bindGroupCache;
    }

    @Override
    public void setPipeline(@NonNull RenderPipeline pipeline) {
        this.rebuildBindGroups = true;
        WgpuCompiledRenderPipeline wgpuPipeline = WgpuCompiledRenderPipeline.wgpuRenderPipelines.computeIfAbsent(pipeline, p -> new WgpuCompiledRenderPipeline(this.device, p, this.device.getDefaultShaderSource()));
        MemorySegment nativePipeline = this.wantsDepth ? wgpuPipeline.getPipelineWithDepth() : wgpuPipeline.getPipelineWithoutDepth();
        WM.bind_render_pipeline_to_pass(nativePass, nativePipeline);
        this.activePipeline = nativePipeline;
    }

    @Override
    public void bindTexture(@NonNull String name, @org.jspecify.annotations.Nullable GpuTextureView textureView, @org.jspecify.annotations.Nullable GpuSampler sampler) {
        try(var arena = Arena.ofConfined()) {
            this.rebuildBindGroups = true;
            WM.bind_texture_and_sampler(this.bindingBuilder, arena.allocateFrom(name), ((WgpuTextureView) textureView).getNative(), ((WgpuSampler) sampler).getNativeSampler());
        }
    }


    @Override
    public void setUniform(@NonNull String name, @NonNull GpuBuffer buffer) {
        this.setUniform(name, buffer.slice());
    }

    @Override
    public void setUniform(@NonNull String name, @NonNull GpuBufferSlice slice) {
        this.rebuildBindGroups = true;
        try(var arena = Arena.ofConfined()) {
            WM.bind_buffer(this.bindingBuilder, arena.allocateFrom(name), ((WgpuBuffer) slice.buffer()).getNativeBuffer(), slice.offset(), Mth.roundToward(slice.length(), 16));
        }
    }

    @Override
    public void enableScissor(int x, int y, int width, int height) {

    }

    @Override
    public void disableScissor() {

    }

    @Override
    public void setVertexBuffer(int slot, @Nullable GpuBufferSlice vertexBuffer) {
        WM.set_vertex_buffer(this.nativePass, slot, ((WgpuBuffer) vertexBuffer.buffer()).getNativeBuffer(), vertexBuffer.offset(), vertexBuffer.length());
    }

    @Override
    public void setIndexBuffer(GpuBuffer indexBuffer, IndexType indexType) {
        WM.set_index_buffer(this.nativePass, ((WgpuBuffer) indexBuffer).getNativeBuffer(), indexType == IndexType.INT);
    }

    @Override
    public void drawIndexed(int indexCount, int instanceCount, int firstIndex, int vertexOffset, int firstInstance) {
        MemorySegment bindGroups = this.buildBindGroups(this.activePipeline);
        WM.draw_indexed(this.nativePass, bindGroups, indexCount, instanceCount, firstIndex, vertexOffset, firstInstance);
    }

    @Override
    public void multiDrawIndexed(IntBuffer drawParameters, int instanceCount, int firstInstance, int drawCount) {

    }

    @Override
    public void multiDrawIndexed(PointerBuffer firstIndexOffsets, IntBuffer indexCounts, IntBuffer vertexOffsets, int drawCount) {

    }

    @Override
    public void drawIndexedIndirect(GpuBufferSlice commands, int drawCount) {
        WgpuMcMod.LOGGER.warn("tried to draw indexed indiret");
    }

    @Override
    public <T> void drawMultipleIndexed(Collection<RenderPass.Draw<T>> draws, @Nullable GpuBuffer defaultIndexBuffer, @Nullable IndexType defaultIndexType, Collection<String> dynamicUniforms, T uniformArgument) {
        for (RenderPass.Draw<T> draw : draws) {
            BiConsumer<T, RenderPass.UniformUploader> uniformUploaderConsumer = draw.uniformUploaderConsumer();
            if (uniformUploaderConsumer != null) {
                uniformUploaderConsumer.accept(uniformArgument, this::setUniform);
            }

            assert draw.indexBuffer() != null || defaultIndexBuffer != null;

            assert draw.indexType() != null || defaultIndexType != null;

            this.setIndexBuffer(draw.indexBuffer() == null ? defaultIndexBuffer : draw.indexBuffer(), draw.indexType() == null ? defaultIndexType : draw.indexType());
            this.setVertexBuffer(draw.slot(), draw.vertexBuffer().slice());
            this.drawIndexed(draw.indexCount(), 1, draw.firstIndex(), draw.baseVertex(), 0);
        }
    }

    @Override
    public void draw(int vertexCount, int instanceCount, int firstVertex, int firstInstance) {
        MemorySegment bindGroups = this.buildBindGroups(activePipeline);
        WM.draw(this.nativePass, bindGroups, vertexCount, instanceCount, firstVertex, firstInstance);
    }

    @Override
    public void multiDraw(IntBuffer drawParameters, int instanceCount, int firstInstance, int drawCount) {
        WgpuMcMod.LOGGER.warn("tried to multi draw");
    }

    @Override
    public void multiDraw(IntBuffer firstVertices, IntBuffer vertexCounts, int drawCount) {
        WgpuMcMod.LOGGER.warn("tried to multi draw");
    }

    @Override
    public void drawIndirect(GpuBufferSlice commands, int drawCount) {
        MemorySegment bindGroups = this.buildBindGroups(this.activePipeline);
        WM.draw_indirect(this.nativePass, bindGroups, ((WgpuBuffer) commands.buffer()).getNativeBuffer(), commands.offset());
    }

    @Override
    public void writeTimestamp(GpuQueryPool pool, int index) {

    }

    @Override
    public void close() throws IOException{
        if(!closed.compareAndExchange(false, true)) {
            WM.drop_render_pass(this.nativePass);
            WM.drop_binding_builder(this.bindingBuilder);
        } else {
            throw new IOException("Already closed");
        }
    }
}
