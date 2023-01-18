package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.util.VideoMode;
import net.minecraft.client.util.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.io.InputStream;
import java.util.Optional;

@Mixin(Window.class)
public class WindowMixin {


    @Shadow private double scaleFactor;

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setVsync(boolean vsync) {
        
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setRawMouseMotion(boolean rawMouseMotion) {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void logOnGlError() {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setIcon(InputStream icon16, InputStream icon32) {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public boolean shouldClose() {
        //TODO
        return false;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getRefreshRate() {
        //TODO
        return 60;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void swapBuffers() {
        Tessellator.getInstance().getBuffer().clear();
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getWidth() {
        return Wgpu.INITIALIZED ? Wgpu.windowWidth : 1280;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getHeight() {
        return Wgpu.INITIALIZED ? Wgpu.windowHeight : 720;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getFramebufferWidth() {
        return this.getWidth();
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getFramebufferHeight() {
        return this.getHeight();
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getScaledWidth() {
        return (int) ((double)this.getWidth() / this.scaleFactor);
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getScaledHeight() {
        return (int) ((double)this.getHeight() / this.scaleFactor);
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setWindowedSize(int width, int height) {
        //TODO
        //WgpuNative.setWindowedSize(width, height);
    }


    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int calculateScaleFactor(int guiScale, boolean forceUnicodeFont) {
        if(guiScale < 1) return 1;
        return guiScale;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public Optional<VideoMode> getVideoMode() {
        if(Wgpu.INITIALIZED) {
            return VideoMode.fromString(WgpuNative.getVideoMode());
        } else {
            return VideoMode.fromString("1920x1080@60:8");
        }
    }

}
