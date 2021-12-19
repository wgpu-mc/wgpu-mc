package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.util.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.io.InputStream;

@Mixin(Window.class)
public class WindowMixin {

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
        return WgpuNative.getWindowWidth();
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int getHeight() {
        return WgpuNative.getWindowHeight();
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

}
