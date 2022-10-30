package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.Tessellator;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.function.Supplier;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Shadow @Nullable private static Thread renderThread;

    @Shadow @Nullable private static Thread gameThread;

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static String getApiDescription() {
        return "wgpu-mc 0.1";
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static String getBackendDescription() {
        return "wgpu 0.14";
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static void flipFrame(long window) {
        Tessellator.getInstance().getBuffer().clear();
        //TODO: events
    }

//    /**
//     * @author wgpu-mc
//     */
//    @Overwrite(remap = false)
//    public static void initGameThread(boolean assertNotRenderThread) {
//        renderThread = Thread.currentThread();
//    }

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static void initRenderer(int debugVerbosity, boolean debugSync) {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static void limitDisplayFPS(int fps) {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static boolean isOnRenderThread() {
        return true;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite(remap = false)
    public static int maxSupportedTextureSize() {
        return 2048; //Probably
//        return WgpuNative.getMaxTextureSize();
    }

//    /**
//     * @author wgpu-mc
//     */
//    @Overwrite(remap = false)
//    public static void _setShaderTexture(int bind_slot, int texId) {
//        WgpuNative.attachTextureBindGroup(texId);
//    }

}
