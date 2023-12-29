package dev.birb.wgpu.mixin;

import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.render.Wgpu;
import net.minecraft.client.gui.hud.DebugHud;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;

@Mixin(DebugHud.class)
public class DebugHudMixin {

    @Inject(method = "getRightText", at = @At("RETURN"))
    public void getRightText(CallbackInfoReturnable<List<String>> cir) {
        if(WgpuMcMod.ENTRIES > 0) {
            cir.getReturnValue().add("[Electrum] texSubImage2D call count: " + Wgpu.getTimesTexSubImageCalled());
            cir.getReturnValue().add("[Electrum] avg uploading entities: " + (WgpuMcMod.TIME_SPENT_ENTITIES / WgpuMcMod.ENTRIES) + "ns");
        }
    }

}
