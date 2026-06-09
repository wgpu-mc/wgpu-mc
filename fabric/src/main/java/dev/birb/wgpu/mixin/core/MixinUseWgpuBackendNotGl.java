package dev.birb.wgpu.mixin.core;

import com.mojang.blaze3d.systems.GpuBackend;
import dev.birb.wgpu.backend.WgpuBackend;
import net.minecraft.client.Minecraft;
import net.minecraft.client.PreferredGraphicsApi;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Minecraft.class)
public class MixinUseWgpuBackendNotGl {

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/PreferredGraphicsApi;getBackendsToTry()[Lcom/mojang/blaze3d/systems/GpuBackend;"))
    public GpuBackend[] forcePreferredApiWgpu(PreferredGraphicsApi instance) {

        return new GpuBackend[] { new WgpuBackend() };
    }


}
