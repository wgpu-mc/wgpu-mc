package dev.birb.wgpu.backend;

import com.mojang.blaze3d.GLFWErrorCapture;
import com.mojang.blaze3d.shaders.GpuDebugOptions;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.systems.BackendCreationException;
import com.mojang.blaze3d.systems.GpuBackend;
import com.mojang.blaze3d.systems.GpuDevice;
import dev.birb.wgpu.rust.WgpuNative;
import org.jspecify.annotations.NonNull;
import org.lwjgl.glfw.GLFW;

public class WgpuBackend implements GpuBackend {

    static {
        WgpuNative.loadWm();
    }

    @Override
    public @NonNull String getName() {
        return "wgpu";
    }

    @Override
    public void setWindowHints() {
        GLFW.glfwWindowHint(GLFW.GLFW_CLIENT_API, GLFW.GLFW_NO_API);
    }

    @Override
    public void handleWindowCreationErrors(GLFWErrorCapture.@NonNull Error error) {

    }

    @Override
    public @NonNull GpuDevice createDevice(long window, @NonNull ShaderSource defaultShaderSource, @NonNull GpuDebugOptions debugOptions, @NonNull Runnable criticalShaderLoader) throws BackendCreationException {
        return new GpuDevice(new WgpuDevice(defaultShaderSource), criticalShaderLoader);
    }

}
