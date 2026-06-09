package dev.birb.wgpu.backend;

import com.mojang.blaze3d.systems.CommandEncoderBackend;
import com.mojang.blaze3d.systems.GpuSurface;
import com.mojang.blaze3d.systems.GpuSurfaceBackend;
import com.mojang.blaze3d.systems.SurfaceException;
import com.mojang.blaze3d.textures.GpuTextureView;
import dev.birb.wm.WM;
import org.jspecify.annotations.NonNull;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWNativeWin32;
import org.lwjgl.glfw.GLFWNativeX11;

import java.lang.foreign.MemorySegment;
import java.util.Collection;
import java.util.List;
import java.util.Objects;

public class WgpuSurface implements GpuSurfaceBackend {

    private final WgpuDevice wgpuDevice;
    private MemorySegment nextTexture;

    public WgpuSurface(WgpuDevice device, long window) {
        long nativeWindowHandle;
        long nativeDisplayHandle;

        this.wgpuDevice = device;

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

        //Internally this attaches to the GpuHandle (wgpu_mc::Gpu)
        WM.create_surface(
                device.getWm(),
                nativeDisplayHandle,
                nativeWindowHandle
        );
    }

    @Override
    public void configure(GpuSurface.@NonNull Configuration config) {
        WM.configure_surface(
                this.wgpuDevice.getWm(),
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
        this.nextTexture = WM.acquire_next_texture(this.wgpuDevice.getWm());
        if(this.nextTexture == MemorySegment.NULL) {
            throw new SurfaceException("Failed to acquire surface texture");
        }
    }

    @Override
    public void blitFromTexture(@NonNull CommandEncoderBackend commandEncoder, @NonNull GpuTextureView textureView) {
        WM.blit_from_texture(
                this.wgpuDevice.getWm(),
                ((WgpuTextureView) textureView).getNative(),
                Objects.requireNonNull(this.nextTexture)
        );
    }

    @Override
    public void present() {
        WM.present_surface(this.wgpuDevice.getWm(), Objects.requireNonNull(this.nextTexture));
        this.nextTexture = null;
    }

    @Override
    public void close() {
        WM.drop_surface(this.wgpuDevice.getWm());
    }

    @Override
    public @NonNull Collection<GpuSurface.PresentMode> supportedPresentModes() {
        return List.of(GpuSurface.PresentMode.values());
    }

}
