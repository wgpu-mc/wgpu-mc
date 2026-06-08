package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.buffers.GpuFence;
import com.mojang.blaze3d.systems.*;
import com.mojang.blaze3d.textures.GpuTexture;
import dev.birb.wm.WM;
import lombok.Getter;
import org.joml.Vector4fc;
import org.jspecify.annotations.NonNull;
import org.jspecify.annotations.Nullable;

import java.io.IOException;
import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.Objects;
import java.util.concurrent.atomic.AtomicBoolean;

public class WgpuCommandEncoder implements CommandEncoderBackend {

    @Getter
    private final MemorySegment nativeCommandEncoder;
    private final AtomicBoolean closed = new AtomicBoolean();

    private final WgpuTransientMemory transientMemory;

    @Nullable
    private WgpuRenderPass renderPass;

    private final WgpuDevice device;

    WgpuCommandEncoder(WgpuDevice device) {
        this.device = device;
        nativeCommandEncoder = WM.create_command_encoder(device.getWm());
        this.transientMemory = new WgpuTransientMemory(device, this);
    }

    @Override
    public void submit() {
        if(!this.closed.compareAndExchange(false, true)) {
            WM.submit_command_encoder(
                    this.device.getWm(),
                    this.nativeCommandEncoder
            );

            try {
                this.transientMemory.close();
            } catch (IOException e) {
                throw new RuntimeException(e);
            }

            return;
        }

        throw new IllegalStateException("Submitting an encoder twice");
    }

    private void flush() {
        WM.flush_encoder(device.getWm(), this.nativeCommandEncoder);
    }

    @Override
    public @NonNull TransientMemory transientMemory() {
        return transientMemory;
    }

    @Override
    public @NonNull RenderPassBackend createRenderPass(@NonNull RenderPassDescriptor descriptor) {
        if(closed.get()) throw new IllegalStateException();
        this.renderPass = new WgpuRenderPass(this.device, this, descriptor);
        return this.renderPass;
    }

    @Override
    public void submitRenderPass() {
        WM.drop_render_pass(Objects.requireNonNull(this.renderPass).getNativePass());
        WM.flush_encoder(device.getWm(), this.nativeCommandEncoder);
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
                this.device.getWm(),
                ((WgpuBuffer) destination.buffer()).getNativeBuffer(),
                destination.offset(),
                data.remaining(),
                MemorySegment.ofBuffer(data)
        );
        this.flush();
    }

    @Override
    public void copyToBuffer(@NonNull GpuBufferSlice source, @NonNull GpuBufferSlice target) {
        assert source.length() + target.offset() < target.buffer().size();

        WM.copy_buffer_to_buffer(
                this.device.getWm(),
                nativeCommandEncoder,
                ((WgpuBuffer) source.buffer()).getNativeBuffer(),
                ((WgpuBuffer) target.buffer()).getNativeBuffer(),
                source.offset(),
                target.offset(),
                source.length()
        );
        this.flush();
    }

    @Override
    public void writeToTexture(@NonNull GpuTexture destination, @NonNull ByteBuffer source, int mipLevel, int depthOrLayer, int destX, int destY, int width, int height) {
        WM.write_to_texture(
                device.getWm(),
                ((WgpuTexture) destination).texture,
                MemorySegment.ofBuffer(source),
                source.remaining(),
                mipLevel,
                depthOrLayer,
                destX,
                destY,
                width,
                height
        );
        this.flush();
    }

    @Override
    public void copyBufferToTexture(@NonNull GpuBufferSlice source, int sourceX, int sourceY, int sourceWidth, int sourceHeight, @NonNull GpuTexture destination, int destinationX, int destinationY, int copyWidth, int copyHeight, int mipLevel, int arrayLayer) {
        WM.copy_buffer_to_texture(
                this.getNativeCommandEncoder(),
                ((WgpuBuffer) source.buffer()).getNativeBuffer(),
                source.offset(),
                source.offset() + source.length(),
                sourceX,
                sourceY,
                sourceWidth,
                sourceHeight,
                ((WgpuTexture) destination).texture,
                destinationX,
                destinationY,
                copyWidth,
                copyHeight,
                mipLevel,
                arrayLayer
        );
        this.flush();
    }

    @Override
    public void copyTextureToBuffer(@NonNull GpuTexture source, @NonNull GpuBuffer destination, long offset, @NonNull Runnable callback, int mipLevel) {

    }

    @Override
    public void copyTextureToBuffer(@NonNull GpuTexture source, @NonNull GpuBuffer destination, long offset, @NonNull Runnable callback, int mipLevel, int x, int y, int width, int height) {
        WM.copy_texture_to_buffer(nativeCommandEncoder, ((WgpuTexture) source).texture, ((WgpuBuffer) destination).getNativeBuffer(), offset, mipLevel, x, y, width, height);
        this.flush();
    }

    @Override
    public void copyTextureToTexture(@NonNull GpuTexture source, @NonNull GpuTexture destination, int mipLevel, int destX, int destY, int sourceX, int sourceY, int width, int height) {
        WM.copy_texture_to_texture(nativeCommandEncoder, ((WgpuTexture) source).texture, ((WgpuTexture) destination).texture, mipLevel, destX, destY, sourceX, sourceY, width, height);
        this.flush();
    }


    @Override
    public @NonNull GpuFence createFence() {
        return new GpuFence() {
            @Override
            public void close() {

            }

            @Override
            public boolean awaitCompletion(long timeoutNS) {
                return true;
            }
        };
    }

    @Override
    public void writeTimestamp(@NonNull GpuQueryPool pool, int index) {

    }

}
