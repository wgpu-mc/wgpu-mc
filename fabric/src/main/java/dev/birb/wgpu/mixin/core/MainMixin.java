package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.main.Main;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Main.class)
public class MainMixin {

    @Redirect(method = "main", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/MinecraftClient;isRunning()Z"))
    private static boolean redirectIsRunning(MinecraftClient instance) {
        //Block until the game is initialized enough for wgpu-mc to kick in
        while(!Wgpu.MAY_INITIALIZE) {}

        Wgpu.initRenderer();

        //Never actually reached
        return true;
    }

}
