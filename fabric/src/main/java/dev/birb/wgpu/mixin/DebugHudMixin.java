package dev.birb.wgpu.mixin;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.gui.hud.DebugHud;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;

@Mixin(DebugHud.class)
public class DebugHudMixin {

    @Inject(method = "getRightText", cancellable = true, at = @At("RETURN"))
    public void getRightText(CallbackInfoReturnable<List<String>> cir) {
        cir.getReturnValue().add("[Electrum] texSubImage2D call count: " + Wgpu.timesTexSubImageCalled);
        cir.getReturnValue().add("[Electrum] Frametime: " + (WgpuNative.getFrameTime() / 1000) + "us");
//        cir.getReturnValue().add("[Electrum] Draw calls: " + Wgpu.timesTexSubImageCalled);
    }

}
