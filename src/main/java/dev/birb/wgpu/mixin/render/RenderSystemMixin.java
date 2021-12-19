package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.Tessellator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.function.Supplier;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Inject(method = "getApiDescription", at = @At("HEAD"), cancellable = true)
    private static void getApiDescription(CallbackInfoReturnable<String> cir) {
        cir.setReturnValue("wgpu-mc 0.1");
    }

    @Inject(method = "getBackendDescription", at = @At("HEAD"), cancellable = true)
    private static void getBackendDescription(CallbackInfoReturnable<String> cir) {
//        cir.setReturnValue(WgpuNative.getBackend());
        cir.setReturnValue("wgpu-mc (Wgpu 0.11)");
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static void assertThread(Supplier<Boolean> check) {
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static void flipFrame(long window) {
        Tessellator.getInstance().getBuffer().clear();
        //TODO: events
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static void initRenderer(int debugVerbosity, boolean debugSync) {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static void limitDisplayFPS(int fps) {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static boolean isOnRenderThread() {
        return true;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static int maxSupportedTextureSize() {
        return 2048; //Probably
//        return WgpuNative.getMaxTextureSize();
    }

}
