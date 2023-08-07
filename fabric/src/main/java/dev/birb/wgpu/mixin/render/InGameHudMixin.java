package dev.birb.wgpu.mixin.render;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class InGameHudMixin {

    @Inject(method = "renderVignetteOverlay", cancellable = true, at = @At("HEAD"))
    public void pleaseDoNotTheVignette(DrawContext context, Entity entity, CallbackInfo ci) {
        ci.cancel();
    }

}
