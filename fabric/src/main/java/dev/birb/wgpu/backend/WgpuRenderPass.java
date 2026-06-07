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
import dev.birb.wm.*;
import org.jspecify.annotations.NonNull;
import org.jspecify.annotations.Nullable;
import org.lwjgl.PointerBuffer;

import java.lang.foreign.*;
import java.nio.IntBuffer;
import java.util.Collection;
import java.util.function.Supplier;

public class WgpuRenderPass implements RenderPassBackend {

    private final MemorySegment nativePass;
    private final WgpuDevice device;
    private static final MemoryLayout vec4fLayout = MemoryLayout.sequenceLayout(
            4, ValueLayout.JAVA_FLOAT
    );

    private static final MemoryLayout attachmentDescriptorLayout = MemoryLayout.structLayout(
            AddressLayout.ADDRESS.withName("texture_view"),
            AddressLayout.ADDRESS.withName("clear_value")
    );

    private static final MemoryLayout renderPassLayout = MemoryLayout.structLayout(
            AddressLayout.ADDRESS.withName("attachments").withTargetLayout(attachmentDescriptorLayout),
            AddressLayout.JAVA_LONG.withName("attachments#count"),
            AddressLayout.ADDRESS.withName("depth_attachment")
    );


    public WgpuRenderPass(WgpuDevice device, WgpuCommandEncoder encoder, RenderPassDescriptor descriptor) {
        this.device = device;

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

                BlazeAttachmentDescriptor__________f32__________4.texture_view(attachmentSeg, view.getNativeView());
                BlazeAttachmentDescriptor__________f32__________4.clear_value(attachmentSeg, clearValRaw);
            }

            MemorySegment depthAttachment = MemorySegment.NULL;

            if(descriptor.depthAttachment() != null) {
                depthAttachment = BlazeAttachmentDescriptor_f64.allocate(arena);
                var depthView = ((WgpuTextureView) descriptor.depthAttachment().textureView()).getNativeView();

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

    @Override
    public void setPipeline(@NonNull RenderPipeline pipeline) {
        WgpuCompiledRenderPipeline wgpuPipeline = WgpuCompiledRenderPipeline.wgpuRenderPipelines.computeIfAbsent(pipeline, p -> new WgpuCompiledRenderPipeline(this.device, p, this.device.getDefaultShaderSource()));
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
    public void setVertexBuffer(int slot, @Nullable GpuBufferSlice vertexBuffer) {

    }

    @Override
    public void setIndexBuffer(GpuBuffer indexBuffer, IndexType indexType) {

    }

    @Override
    public void drawIndexed(int indexCount, int instanceCount, int firstIndex, int vertexOffset, int firstInstance) {

    }

    @Override
    public void multiDrawIndexed(IntBuffer drawParameters, int instanceCount, int firstInstance, int drawCount) {

    }

    @Override
    public void multiDrawIndexed(PointerBuffer firstIndexOffsets, IntBuffer indexCounts, IntBuffer vertexOffsets, int drawCount) {

    }

    @Override
    public void drawIndexedIndirect(GpuBufferSlice commands, int drawCount) {

    }

    @Override
    public <T> void drawMultipleIndexed(Collection<RenderPass.Draw<T>> draws, @Nullable GpuBuffer defaultIndexBuffer, @Nullable IndexType defaultIndexType, Collection<String> dynamicUniforms, T uniformArgument) {

    }

    @Override
    public void draw(int vertexCount, int instanceCount, int firstVertex, int firstInstance) {

    }

    @Override
    public void multiDraw(IntBuffer drawParameters, int instanceCount, int firstInstance, int drawCount) {

    }

    @Override
    public void multiDraw(IntBuffer firstVertices, IntBuffer vertexCounts, int drawCount) {

    }

    @Override
    public void drawIndirect(GpuBufferSlice commands, int drawCount) {

    }

    @Override
    public void writeTimestamp(GpuQueryPool pool, int index) {

    }

}
