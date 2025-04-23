package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.buffers.GpuFence;
import com.mojang.blaze3d.systems.CommandEncoder;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.textures.GpuTexture;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.OptionalDouble;
import java.util.OptionalInt;
import java.util.function.Supplier;

public class WgpuCommandEncoder implements CommandEncoder {

    private long ptr;

    public WgpuCommandEncoder() {
        this.ptr = WgpuNative.createCommandEncoder();
    }

    @Override
    public RenderPass createRenderPass(Supplier<String> supplier, GpuTexture gpuTexture, OptionalInt optionalInt) {
        return new WgpuRenderPass();
    }

    @Override
    public RenderPass createRenderPass(Supplier<String> supplier, GpuTexture gpuTexture, OptionalInt optionalInt, @Nullable GpuTexture gpuTexture2, OptionalDouble optionalDouble) {
        return new WgpuRenderPass();
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
        return new WgpuBuffer.WgpuMappedView(((WgpuBuffer) buffer).getMap());
    }

    @Override
    public GpuBuffer.MappedView mapBuffer(GpuBufferSlice slice, boolean read, boolean write) {
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

    }

    @Override
    public GpuFence createFence() {
        return null;
    }
}
