package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.render.Wgpu;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.main.Main;
import org.checkerframework.checker.units.qual.A;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Main.class)
public class MainMixin {

    @Inject(method = "main", at = @At("HEAD"))
    private static void preInit(String[] args, CallbackInfo ci) {
        Wgpu.preInit("Minecraft");
    }

    @Redirect(method = "main", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/MinecraftClient;isRunning()Z"))
    private static boolean redirectIsRunning(MinecraftClient instance) {
        //Block until the game is initialized enough for wgpu-mc to kick in
        while(!Wgpu.MAY_INITIALIZE) {}

        Wgpu.client.set(instance);
        System.out.println(Wgpu.client.get());
        Wgpu.startRendering();

        //Never actually reached
        return true;
    }

}
