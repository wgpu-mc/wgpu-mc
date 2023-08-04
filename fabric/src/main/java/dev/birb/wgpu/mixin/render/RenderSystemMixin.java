package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.render.Tessellator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {
    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static String getApiDescription() {
        return "wgpu-mc 0.1";
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static String getBackendDescription() {
        return "wgpu 0.14";
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void flipFrame(long window) {
        Tessellator.getInstance().getBuffer().clear();
        //TODO: events
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void initRenderer(int debugVerbosity, boolean debugSync) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void limitDisplayFPS(int fps) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static boolean isOnRenderThread() {
        return true;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int maxSupportedTextureSize() {
        return 8192; // 8192 is the default. we can't get this value before the renderer is initialized.
    }
}
