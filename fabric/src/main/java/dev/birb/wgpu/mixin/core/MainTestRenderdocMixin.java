package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.fabricmc.loader.impl.launch.knot.KnotClient;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.main.Main;
import org.checkerframework.checker.units.qual.A;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Main.class)
public class MainTestRenderdocMixin {

    @Inject(method = "main", at = @At("HEAD"))
    private static void redirectIsRunning(String[] args, CallbackInfo ci) {
        Wgpu.linkRenderDoc();
    }

}
