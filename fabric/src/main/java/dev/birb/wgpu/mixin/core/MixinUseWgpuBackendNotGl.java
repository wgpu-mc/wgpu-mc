package dev.birb.wgpu.mixin.core;

import com.mojang.blaze3d.systems.GpuBackend;
import dev.birb.wgpu.backend.WgpuBackend;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

@Mixin(Minecraft.class)
public class MixinUseWgpuBackendNotGl {

    @ModifyVariable(method = "<init>(Lnet/minecraft/client/main/GameConfig;)V", at = @At("STORE"), name = "backends")
    public GpuBackend[] changeBackendsArray(GpuBackend[] backends) {
        return new GpuBackend[] { new WgpuBackend() };
    }

}
