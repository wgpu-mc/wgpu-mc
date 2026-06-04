package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.buffers.GpuFence;
import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.systems.CommandEncoderBackend;
import com.mojang.blaze3d.systems.GpuQuery;
import com.mojang.blaze3d.systems.RenderPassBackend;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.GpuTextureView;
import org.jspecify.annotations.Nullable;

import java.nio.ByteBuffer;
import java.util.OptionalDouble;
import java.util.OptionalInt;
import java.util.function.Supplier;

public class WgpuCommandEncoder implements CommandEncoderBackend {

    private long ptr;


    @Override
    public RenderPassBackend createRenderPass(Supplier<String> label, GpuTextureView colorTexture, OptionalInt clearColor) {
        return null;
    }

    @Override
    public RenderPassBackend createRenderPass(Supplier<String> label, GpuTextureView colorTexture, OptionalInt clearColor, @Nullable GpuTextureView depthTexture, OptionalDouble clearDepth) {
        return null;
    }

    @Override
    public boolean isInRenderPass() {
        return false;
    }

    @Override
    public void clearColorTexture(GpuTexture colorTexture, int clearColor) {

    }

    @Override
    public void clearColorAndDepthTextures(GpuTexture colorTexture, int clearColor, GpuTexture depthTexture, double clearDepth) {

    }

    @Override
    public void clearColorAndDepthTextures(GpuTexture colorTexture, int clearColor, GpuTexture depthTexture, double clearDepth, int regionX, int regionY, int regionWidth, int regionHeight) {

    }

    @Override
    public void clearDepthTexture(GpuTexture depthTexture, double clearDepth) {

    }

    @Override
    public void writeToBuffer(GpuBufferSlice destination, ByteBuffer data) {

    }

    @Override
    public GpuBuffer.MappedView mapBuffer(GpuBufferSlice buffer, boolean read, boolean write) {
        return null;
    }

    @Override
    public void copyToBuffer(GpuBufferSlice source, GpuBufferSlice target) {

    }

    @Override
    public void writeToTexture(GpuTexture destination, NativeImage source, int mipLevel, int depthOrLayer, int destX, int destY, int width, int height, int sourceX, int sourceY) {

    }

    @Override
    public void writeToTexture(GpuTexture destination, ByteBuffer source, NativeImage.Format format, int mipLevel, int depthOrLayer, int destX, int destY, int width, int height) {

    }

    @Override
    public void copyTextureToBuffer(GpuTexture source, GpuBuffer destination, long offset, Runnable callback, int mipLevel) {

    }

    @Override
    public void copyTextureToBuffer(GpuTexture source, GpuBuffer destination, long offset, Runnable callback, int mipLevel, int x, int y, int width, int height) {

    }

    @Override
    public void copyTextureToTexture(GpuTexture source, GpuTexture destination, int mipLevel, int destX, int destY, int sourceX, int sourceY, int width, int height) {

    }

    @Override
    public void presentTexture(GpuTextureView texture) {

    }

    @Override
    public GpuFence createFence() {
        return null;
    }

    @Override
    public GpuQuery timerQueryBegin() {
        return null;
    }

    @Override
    public void timerQueryEnd(GpuQuery query) {

    }
}
