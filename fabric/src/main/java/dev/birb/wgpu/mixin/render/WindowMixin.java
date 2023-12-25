package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.util.Icons;
import net.minecraft.client.util.VideoMode;
import net.minecraft.client.util.Window;
import net.minecraft.resource.ResourcePack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.Optional;

@Mixin(Window.class)
public class WindowMixin {


    @Shadow private double scaleFactor;

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void setVsync(boolean vsync) {
        
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void setRawMouseMotion(boolean rawMouseMotion) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void logOnGlError() {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void setIcon(ResourcePack resourcePack, Icons icons) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public boolean shouldClose() {
        //TODO
        return false;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getRefreshRate() {
        //TODO
        return 60;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void swapBuffers() {
        Tessellator.getInstance().getBuffer().clear();
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getWidth() {
        return Wgpu.isInitialized() ? Wgpu.getWindowWidth() : 1280;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getHeight() {
        return Wgpu.isInitialized() ? Wgpu.getWindowHeight() : 720;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getFramebufferWidth() {
        return this.getWidth();
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getFramebufferHeight() {
        return this.getHeight();
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getScaledWidth() {
        return (int) (this.getWidth() / this.scaleFactor);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int getScaledHeight() {
        return (int) (this.getHeight() / this.scaleFactor);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public void setWindowedSize(int width, int height) {
        //TODO
    }


    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public int calculateScaleFactor(int guiScale, boolean forceUnicodeFont) {
        return Math.max(guiScale, 1);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
//    @Overwrite
//    public Optional<VideoMode> getVideoMode() {
//        if (Wgpu.isInitialized()) {
//            return VideoMode.fromString(WgpuNative.getVideoMode());
//        } else {
//            return VideoMode.fromString("1920x1080@60:8");
//        }
//    }

}
