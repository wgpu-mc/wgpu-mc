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
import dev.birb.wm.WM;
import org.jspecify.annotations.NonNull;
import org.jspecify.annotations.Nullable;

import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.util.OptionalDouble;
import java.util.OptionalInt;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Supplier;

public class WgpuCommandEncoder implements CommandEncoderBackend {

    private final MemorySegment nativeCommandEncoder;
    private final AtomicBoolean closed = new AtomicBoolean();

    private final WgpuDevice device;

    WgpuCommandEncoder(WgpuDevice device) {
        this.device = device;
        nativeCommandEncoder = WM.create_command_encoder();
    }

    @Override
    public @Nullable RenderPassBackend createRenderPass(@NonNull Supplier<String> label, @NonNull GpuTextureView colorTexture, @NonNull OptionalInt clearColor) {
        if(closed.get()) throw new IllegalStateException();
        return new WgpuRenderPass(nativeCommandEncoder, label.get(), (WgpuTextureView) colorTexture, clearColor, device);
    }

    @Override
    public @NonNull RenderPassBackend createRenderPass(@NonNull Supplier<String> label, @NonNull GpuTextureView colorTexture, @NonNull OptionalInt clearColor, @Nullable GpuTextureView depthTexture, @NonNull OptionalDouble clearDepth) {
        if(closed.get()) throw new IllegalStateException();
        return new WgpuRenderPass(nativeCommandEncoder, label.get(), (WgpuTextureView) colorTexture, clearColor, (WgpuTextureView) depthTexture, clearDepth, device);
    }

    @Override
    public boolean isInRenderPass() {
        return false;
    }

    @Override
    public void clearColorTexture(@NonNull GpuTexture colorTexture, int clearColor) {

    }

    @Override
    public void clearColorAndDepthTextures(@NonNull GpuTexture colorTexture, int clearColor, @NonNull GpuTexture depthTexture, double clearDepth) {

    }

    @Override
    public void clearColorAndDepthTextures(@NonNull GpuTexture colorTexture, int clearColor, @NonNull GpuTexture depthTexture, double clearDepth, int regionX, int regionY, int regionWidth, int regionHeight) {

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
    public GpuBuffer.@NonNull MappedView mapBuffer(@NonNull GpuBufferSlice buffer, boolean read, boolean write) {
        return new WgpuBuffer.WgpuMappedView(buffer.length(), (WgpuBuffer) buffer.buffer());
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
    public void writeToTexture(@NonNull GpuTexture destination, @NonNull NativeImage source, int mipLevel, int depthOrLayer, int destX, int destY, int width, int height, int sourceX, int sourceY) {

    }

    @Override
    public void writeToTexture(@NonNull GpuTexture destination, @NonNull ByteBuffer source, NativeImage.@NonNull Format format, int mipLevel, int depthOrLayer, int destX, int destY, int width, int height) {

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
    public void presentTexture(@NonNull GpuTextureView texture) {
        if(!closed.compareAndExchange(false, true)) {
            //SAFETY: This function consumes the command encoder, but this can only happen once due to the AtomicBoolean
            WM.present_texture(nativeCommandEncoder, ((WgpuTextureView) texture).nativeView);
        } else {
            throw new IllegalStateException();
        }
    }

    @Override
    public @NonNull GpuFence createFence() {
        return null;
    }

    @Override
    public @NonNull GpuQuery timerQueryBegin() {
        return null;
    }

    @Override
    public void timerQueryEnd(@NonNull GpuQuery query) {

    }
}
