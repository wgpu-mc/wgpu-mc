package dev.birb.wgpu.mixin.core;

import net.minecraft.client.main.Main;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import dev.birb.wgpu.render.Wgpu;

@Mixin(Main.class)
public class MainTestRenderdocMixin {

    @Inject(method = "main", at = @At("HEAD"))
    private static void redirectIsRunning(String[] args, CallbackInfo ci) {
        Wgpu.linkRenderDoc();
    }

}
