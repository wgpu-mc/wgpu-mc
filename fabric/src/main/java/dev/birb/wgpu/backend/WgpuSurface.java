package dev.birb.wgpu.backend;

import com.mojang.blaze3d.systems.CommandEncoderBackend;
import com.mojang.blaze3d.systems.GpuSurface;
import com.mojang.blaze3d.systems.GpuSurfaceBackend;
import com.mojang.blaze3d.systems.SurfaceException;
import com.mojang.blaze3d.textures.GpuTextureView;
import dev.birb.wm.WM;
import lombok.Getter;
import org.jspecify.annotations.NonNull;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWNativeWin32;
import org.lwjgl.glfw.GLFWNativeX11;

import java.lang.foreign.MemorySegment;
import java.util.Collection;
import java.util.List;

public class WgpuSurface implements GpuSurfaceBackend {

    @Getter
    private final MemorySegment nativeSurface;

    @Getter
    private final MemorySegment gpuHandle;
    private final WgpuDevice wgpuDevice;

    public WgpuSurface(WgpuDevice device, MemorySegment gpuHandle, long window) {
        long nativeWindowHandle;
        long nativeDisplayHandle;

        this.wgpuDevice = device;
        this.gpuHandle = gpuHandle;

        if (GLFW.glfwGetPlatform() == GLFW.GLFW_PLATFORM_WIN32) {
            // windows doesn't use display, so 0 is fine
            nativeWindowHandle = GLFWNativeWin32.glfwGetWin32Window(window);
            nativeDisplayHandle = 0;
        } else if (GLFW.glfwGetPlatform() == GLFW.GLFW_PLATFORM_X11) {
            nativeDisplayHandle = GLFWNativeX11.glfwGetX11Display();
            nativeWindowHandle = GLFWNativeX11.glfwGetX11Window(window);
        } else {
            throw new RuntimeException("Platform not supported");
        }

        this.nativeSurface = WM.create_surface(
                gpuHandle,
                nativeDisplayHandle,
                nativeWindowHandle
        );
    }

    @Override
    public void configure(GpuSurface.@NonNull Configuration config) {
        WM.configure_surface(
                this.wgpuDevice.getWmGpuHandle(),
                this.nativeSurface,
                config.width(),
                config.height(),
                //TODO present modes
                0
        );
    }

    @Override
    public boolean isSuboptimal() {
        return false;
    }

    @Override
    public void acquireNextTexture() throws SurfaceException {

    }

    @Override
    public void blitFromTexture(@NonNull CommandEncoderBackend commandEncoder, @NonNull GpuTextureView textureView) {

    }

    @Override
    public void present() {

    }

    @Override
    public void close() {
        WM.drop_surface(this.nativeSurface);
    }

    @Override
    public @NonNull Collection<GpuSurface.PresentMode> supportedPresentModes() {
        return List.of();
    }

}
