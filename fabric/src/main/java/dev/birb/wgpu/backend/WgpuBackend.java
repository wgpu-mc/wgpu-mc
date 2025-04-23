package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.ShaderType;
import com.mojang.blaze3d.systems.CommandEncoder;
import com.mojang.blaze3d.systems.GpuDevice;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.TextureFormat;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.Nullable;

import java.nio.ByteBuffer;
import java.util.List;
import java.util.function.BiFunction;
import java.util.function.Supplier;

public class WgpuBackend implements GpuDevice {

    private final int minUniformOffsetAlignment;
    private final int maxTextureSize;

    public WgpuBackend(long window, long getWindow) {
        int w = MinecraftClient.getInstance().getWindow().getWidth();
        int h = MinecraftClient.getInstance().getWindow().getHeight();
        WgpuNative.createDevice(window, getWindow, w, h);

        this.minUniformOffsetAlignment = WgpuNative.getMinUniformAlignment();
        this.maxTextureSize = WgpuNative.getMaxTextureSize();
    }

    @Override
    public CommandEncoder createCommandEncoder() {
        return new WgpuCommandEncoder();
    }

    @Override
    public GpuTexture createTexture(@Nullable Supplier<String> labelGetter, int i, TextureFormat textureFormat, int height, int mipLevels, int j) {
        return this.createTexture(labelGetter.get(), i, textureFormat, height, mipLevels, j);
    }

    @Override
    public GpuTexture createTexture(@Nullable String label, int usage, TextureFormat textureFormat, int width, int height, int mipLevels) {
        return new WgpuTexture(usage, label,  textureFormat, width, height, mipLevels);
    }

    @Override
    public GpuBuffer createBuffer(@Nullable Supplier<String> labelGetter, int usage, int size) {
        String label = labelGetter.get();
        return new WgpuBuffer(label != null ? label : "<mc buffer>", usage, size);
    }

    @Override
    public GpuBuffer createBuffer(@Nullable Supplier<String> labelGetter, int usage, ByteBuffer data) {
        String label = labelGetter.get();
        return new WgpuBuffer(label != null ? label : "<mc buffer>", usage, data);
    }

    @Override
    public String getImplementationInformation() {
        return "wgpu";
    }

    @Override
    public List<String> getLastDebugMessages() {
        return List.of();
    }

    @Override
    public boolean isDebuggingEnabled() {
        return false;
    }

    @Override
    public String getVendor() {
        return "wgpu";
    }

    @Override
    public String getBackendName() {
        return "wgpu";
    }

    @Override
    public String getVersion() {
        return "22";
    }

    @Override
    public String getRenderer() {
        return "wgpu-mc";
    }

    @Override
    public int getMaxTextureSize() {
        return this.maxTextureSize;
    }

    @Override
    public int getUniformOffsetAlignment() {
        return minUniformOffsetAlignment;
    }

    @Override
    public CompiledRenderPipeline precompilePipeline(RenderPipeline pipeline, @Nullable BiFunction<Identifier, ShaderType, String> sourceRetriever) {
        return null;
    }

    @Override
    public void clearPipelineCache() {

    }

    @Override
    public List<String> getEnabledExtensions() {
        return List.of();
    }

    @Override
    public void close() {

    }
}
