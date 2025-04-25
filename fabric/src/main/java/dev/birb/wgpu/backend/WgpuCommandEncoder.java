package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.buffers.GpuFence;
import com.mojang.blaze3d.systems.CommandEncoder;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.textures.GpuTexture;
import dev.birb.wgpu.rust.WgpuNative;
import lombok.Getter;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.*;
import java.util.function.Supplier;

public class WgpuCommandEncoder implements CommandEncoder {

    private static List<WgpuCommandEncoder> encoders = new ArrayList<>();

    private final List<WgpuRenderPass> renderPasses = new ArrayList<>();
    @Getter
    private final Set<WgpuBuffer> mappedBuffers = new HashSet<>();

    public WgpuCommandEncoder() {
        encoders.add(this);
    }

    @Override
    public RenderPass createRenderPass(Supplier<String> supplier, GpuTexture colorAttachment, OptionalInt optionalInt) {
        WgpuRenderPass pass = new WgpuRenderPass((WgpuTexture) colorAttachment, null);
        renderPasses.add(pass);
        return pass;
    }

    @Override
    public RenderPass createRenderPass(Supplier<String> supplier, GpuTexture colorAttachment, OptionalInt optionalInt, @Nullable GpuTexture depthAttachment, OptionalDouble optionalDouble) {
        WgpuRenderPass pass = new WgpuRenderPass((WgpuTexture) colorAttachment, (WgpuTexture) depthAttachment);
        renderPasses.add(pass);
        return pass;
    }

    @Override
    public void clearColorTexture(GpuTexture texture, int color) {

    }

    @Override
    public void clearColorAndDepthTextures(GpuTexture colorAttachment, int color, GpuTexture depthAttachment, double depth) {

    }

    @Override
    public void clearColorAndDepthTextures(GpuTexture colorAttachment, int color, GpuTexture depthAttachment, double depth, int scissorX, int scissorY, int scissorWidth, int scissorHeight) {

    }

    @Override
    public void clearDepthTexture(GpuTexture texture, double depth) {

    }

    @Override
    public void writeToBuffer(GpuBufferSlice slice, ByteBuffer source) {

    }

    @Override
    public GpuBuffer.MappedView mapBuffer(GpuBuffer buffer, boolean read, boolean write) {
        if(write) this.mappedBuffers.add((WgpuBuffer) buffer);

        ByteBuffer buf = ((WgpuBuffer) buffer).getMap();
        return new WgpuBuffer.WgpuMappedView(buf.slice(0, buf.capacity()));
    }

    @Override
    public GpuBuffer.MappedView mapBuffer(GpuBufferSlice slice, boolean read, boolean write) {
        if(write) this.mappedBuffers.add((WgpuBuffer) slice.buffer());

        return new WgpuBuffer.WgpuMappedView(((WgpuBuffer) slice.buffer()).getMap().slice(slice.offset(), slice.length()));
    }

    @Override
    public void writeToTexture(GpuTexture target, NativeImage source) {

    }

    @Override
    public void writeToTexture(GpuTexture target, NativeImage source, int mipLevel, int intoX, int intoY, int width, int height, int x, int y) {

    }

    @Override
    public void writeToTexture(GpuTexture target, IntBuffer source, NativeImage.Format format, int mipLevel, int intoX, int intoY, int width, int height) {

    }

    @Override
    public void copyTextureToBuffer(GpuTexture target, GpuBuffer source, int offset, Runnable dataUploadedCallback, int mipLevel) {

    }

    @Override
    public void copyTextureToBuffer(GpuTexture target, GpuBuffer source, int offset, Runnable dataUploadedCallback, int mipLevel, int intoX, int intoY, int width, int height) {

    }

    @Override
    public void copyTextureToTexture(GpuTexture target, GpuTexture source, int mipLevel, int intoX, int intoY, int sourceX, int sourceY, int width, int height) {

    }

    @Override
    public void presentTexture(GpuTexture texture) {
        submitAllEncoders();
        WgpuNative.presentTexture(((WgpuTexture) texture).getTexture());
    }

    private static void submitAllEncoders() {
        List<WgpuCommandEncoder> oldEncoders = WgpuCommandEncoder.encoders;
        ByteBuffer toSubmit = MemoryUtil.memCalloc(oldEncoders.size() * 32);
        WgpuCommandEncoder.encoders = new ArrayList<>();

        for (WgpuCommandEncoder encoder : oldEncoders) {
            ByteBuffer renderPassQueue = MemoryUtil.memCalloc(32 * encoder.renderPasses.size());
            ByteBuffer buffersToWrite = MemoryUtil.memCalloc(encoder.getMappedBuffers().size() * 16);

            for(WgpuRenderPass pass : encoder.renderPasses) {
                renderPassQueue.putLong(pass.getTarget().getTexture());
                renderPassQueue.putLong(
                        pass.getDepth() != null ? pass.getDepth().getTexture() : 0L
                );
                renderPassQueue.putLong(MemoryUtil.memAddress0(pass.getCommands()));
                renderPassQueue.putInt(pass.getCommandCount());

                //Padding
                renderPassQueue.position(renderPassQueue.position() + 4);
            }

            for(WgpuBuffer buffer : encoder.getMappedBuffers()) {
                buffersToWrite.putLong(buffer.getWgpuBuffer());
                buffersToWrite.putLong(MemoryUtil.memAddress0(buffer.getMap()));
            }

            toSubmit.putLong(MemoryUtil.memAddress0(renderPassQueue));
            toSubmit.putLong(encoder.renderPasses.size());
            toSubmit.putLong(MemoryUtil.memAddress0(buffersToWrite));
            toSubmit.putLong(encoder.getMappedBuffers().size());
        }

        WgpuNative.submitEncoders(MemoryUtil.memAddress0(toSubmit), oldEncoders.size());
    }

    @Override
    public GpuFence createFence() {
        return null;
    }
}
