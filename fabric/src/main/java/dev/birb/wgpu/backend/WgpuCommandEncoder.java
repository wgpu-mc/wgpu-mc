package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.buffers.GpuFence;
import com.mojang.blaze3d.systems.*;
import com.mojang.blaze3d.textures.GpuTexture;
import dev.birb.wm.WM;
import org.apache.commons.lang3.NotImplementedException;
import org.joml.Vector4fc;
import org.jspecify.annotations.NonNull;

import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuCommandEncoder implements CommandEncoderBackend {

    private final MemorySegment nativeCommandEncoder;
    private final AtomicBoolean closed = new AtomicBoolean();

    private final WgpuDevice device;

    WgpuCommandEncoder(WgpuDevice device) {
        this.device = device;
        nativeCommandEncoder = WM.create_command_encoder();
    }

    @Override
    public void submit() {

    }

    @Override
    public @NonNull TransientMemory transientMemory() {
        throw new NotImplementedException();
    }

    @Override
    public @NonNull RenderPassBackend createRenderPass(@NonNull RenderPassDescriptor descriptor) {
        if(closed.get()) throw new IllegalStateException();
        return new WgpuRenderPass(this.device, nativeCommandEncoder, descriptor);
    }

    @Override
    public void submitRenderPass() {

    }

    @Override
    public void clearColorTexture(@NonNull GpuTexture colorTexture, @NonNull Vector4fc clearColor) {

    }

    @Override
    public void clearColorAndDepthTextures(@NonNull GpuTexture colorTexture, @NonNull Vector4fc clearColor, @NonNull GpuTexture depthTexture, double clearDepth) {

    }

    @Override
    public void clearColorAndDepthTextures(@NonNull GpuTexture colorTexture, @NonNull Vector4fc clearColor, @NonNull GpuTexture depthTexture, double clearDepth, int regionX, int regionY, int regionWidth, int regionHeight) {

    }

    @Override
    public void clearDepthTexture(@NonNull GpuTexture depthTexture, double clearDepth) {

    }

    @Override
    public void writeToBuffer(@NonNull GpuBufferSlice destination, @NonNull ByteBuffer data) {
        WM.write_to_buffer(
                ((WgpuBuffer) destination.buffer()).getNativeBuffer(),
                destination.offset(),
                destination.length(),
                MemorySegment.ofBuffer(data)
        );
    }

    @Override
    public void copyToBuffer(@NonNull GpuBufferSlice source, @NonNull GpuBufferSlice target) {
        assert target.length() >= source.length();

        WM.copy_buffer_to_buffer(
                nativeCommandEncoder,
                ((WgpuBuffer) source.buffer()).getNativeBuffer(),
                ((WgpuBuffer) target.buffer()).getNativeBuffer(),
                source.offset(),
                target.offset(),
                source.length()
        );
    }

    @Override
    public void writeToTexture(@NonNull GpuTexture destination, @NonNull ByteBuffer source, int mipLevel, int depthOrLayer, int destX, int destY, int width, int height) {

    }

    @Override
    public void copyBufferToTexture(@NonNull GpuBufferSlice source, int sourceX, int sourceY, int sourceWidth, int sourceHeight, @NonNull GpuTexture destination, int destinationX, int destinationY, int copyWidth, int copyHeight, int mipLevel, int arrayLayer) {

    }

    @Override
    public void copyTextureToBuffer(@NonNull GpuTexture source, @NonNull GpuBuffer destination, long offset, @NonNull Runnable callback, int mipLevel) {

    }

    @Override
    public void copyTextureToBuffer(@NonNull GpuTexture source, @NonNull GpuBuffer destination, long offset, @NonNull Runnable callback, int mipLevel, int x, int y, int width, int height) {

    }

    @Override
    public void copyTextureToTexture(@NonNull GpuTexture source, @NonNull GpuTexture destination, int mipLevel, int destX, int destY, int sourceX, int sourceY, int width, int height) {

    }


    @Override
    public @NonNull GpuFence createFence() {
        return null;
    }

    @Override
    public void writeTimestamp(@NonNull GpuQueryPool pool, int index) {

    }

}
