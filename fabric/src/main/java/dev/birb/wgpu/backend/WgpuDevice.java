package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.systems.CommandEncoderBackend;
import com.mojang.blaze3d.systems.GpuDeviceBackend;
import com.mojang.blaze3d.textures.*;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wm.WM;
import lombok.Getter;
import org.jetbrains.annotations.Nullable;
import org.jspecify.annotations.NonNull;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWNativeWin32;
import org.lwjgl.glfw.GLFWNativeX11;

import java.nio.ByteBuffer;
import java.util.List;
import java.util.OptionalDouble;
import java.util.function.Supplier;

public class WgpuDevice implements GpuDeviceBackend {

    private final int minUniformOffsetAlignment;
    private final int maxTextureSize;

    @Getter
    private final ShaderSource defaultShaderSource;

    // private final BiFunction<Identifier, ShaderType, String> shaderSourceGetter;

    public WgpuDevice(long window, ShaderSource shaderSource) {
        this.defaultShaderSource = shaderSource;

        int[] w = new int[1];
        int[] h = new int[1];

        GLFW.glfwGetFramebufferSize(window, w, h);
        GLFW.glfwFocusWindow(window);
        GLFW.glfwShowWindow(window);

        GLFW.glfwPollEvents();
        GLFW.glfwPollEvents();

        if (GLFW.glfwGetPlatform() == GLFW.GLFW_PLATFORM_WIN32) {
            // windows doesn't use display, so 0 is fine
            WgpuNative.createDevice(0, GLFWNativeWin32.glfwGetWin32Window(window), w[0], h[0]);
        } else if (GLFW.glfwGetPlatform() == GLFW.GLFW_PLATFORM_X11) {
            WgpuNative.createDevice(GLFWNativeX11.glfwGetX11Display(), GLFWNativeX11.glfwGetX11Window(window), w[0],
                    h[0]);
        } else {
            throw new RuntimeException("Platform not supported");
        }

        this.minUniformOffsetAlignment = WM.min_uniform_offset_alignment();
        this.maxTextureSize = WM.max_texture_size();
    }

    @Override
    public @NonNull CommandEncoderBackend createCommandEncoder() {
        return new WgpuCommandEncoder(this);
    }

    @Override
    public @NonNull GpuSampler createSampler(@NonNull AddressMode addressModeU, @NonNull AddressMode addressModeV,
            @NonNull FilterMode minFilter, @NonNull FilterMode magFilter, int maxAnisotropy,
            @NonNull OptionalDouble maxLod) {
        return new WgpuSampler();
    }

    @Override
    public @NonNull GpuTexture createTexture(@org.jspecify.annotations.Nullable Supplier<String> label,
            @GpuTexture.Usage int usage, @NonNull TextureFormat format, int width, int height, int depthOrLayers,
            int mipLevels) {
        // return this.createTexture(label.get(), usage, format, height, mipLevels, );
        return this.createTexture(label == null ? "<wm/unnamed mc texture>" : label.get(), usage, format, width, height,
                depthOrLayers, mipLevels);
    }

    @Override
    public @NonNull GpuTexture createTexture(@org.jspecify.annotations.Nullable String label,
            @GpuTexture.Usage int usage, @NonNull TextureFormat format, int width, int height, int depthOrLayers,
            int mipLevels) {
        return new WgpuTexture(usage, label, format, width, height, depthOrLayers, mipLevels);
        // return null;
    }

    @Override
    public @NonNull GpuTextureView createTextureView(@NonNull GpuTexture texture) {
        return new WgpuTextureView((WgpuTexture) texture, 1, 1);
    }

    @Override
    public @NonNull GpuTextureView createTextureView(@NonNull GpuTexture texture, int baseMipLevel, int mipLevels) {
        return new WgpuTextureView((WgpuTexture) texture, baseMipLevel, mipLevels);
    }

    @Override
    public @NonNull GpuBuffer createBuffer(@org.jspecify.annotations.Nullable Supplier<String> label,
            @GpuBuffer.Usage int usage, long size) {
        return new WgpuBuffer(label != null ? label.get() : "<wm/unnamed mc buffer>", usage, size);
    }

    @Override
    public @NonNull GpuBuffer createBuffer(@Nullable Supplier<String> labelGetter, int usage,
            @NonNull ByteBuffer data) {
        return new WgpuBuffer(labelGetter != null ? labelGetter.get() : "<wm/unnamed mc buffer>", usage, data);
    }

    @Override
    public @NonNull String getImplementationInformation() {
        return "wgpu";
    }

    @Override
    public @NonNull List<String> getLastDebugMessages() {
        return List.of();
    }

    @Override
    public boolean isDebuggingEnabled() {
        return false;
    }

    @Override
    public @NonNull String getVendor() {
        return "wgpu";
    }

    @Override
    public @NonNull String getBackendName() {
        return "wgpu";
    }

    @Override
    public @NonNull String getVersion() {
        return "22";
    }

    @Override
    public @NonNull String getRenderer() {
        return "electrum";
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
    public @NonNull CompiledRenderPipeline precompilePipeline(@NonNull RenderPipeline pipeline,
            @org.jspecify.annotations.Nullable ShaderSource shaderSource) {
        var source = shaderSource != null ? shaderSource : this.defaultShaderSource;

        return WgpuCompiledRenderPipeline.wgpuRenderPipelines.computeIfAbsent(pipeline,
                p -> new WgpuCompiledRenderPipeline(p, source));
    }

    @Override
    public void clearPipelineCache() {
        WgpuCompiledRenderPipeline.wgpuRenderPipelines.clear();
        WgpuCompiledRenderPipeline.shaderSourceCache.clear();
    }

    @Override
    public @NonNull List<String> getEnabledExtensions() {
        return List.of();
    }

    @Override
    public int getMaxSupportedAnisotropy() {
        return 1;
    }

    @Override
    public void close() {

    }

    @Override
    public void setVsync(boolean enabled) {

    }

    @Override
    public void presentFrame() {

    }

    @Override
    public boolean isZZeroToOne() {
        return false;
    }
}
