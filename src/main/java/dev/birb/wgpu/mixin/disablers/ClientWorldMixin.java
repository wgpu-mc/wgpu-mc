package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.world.ClientWorld;
import net.minecraft.sound.SoundCategory;
import net.minecraft.sound.SoundEvent;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientWorld.class)
public class ClientWorldMixin {

    @Inject(method = "playSound(DDDLnet/minecraft/sound/SoundEvent;Lnet/minecraft/sound/SoundCategory;FFZ)V", cancellable = true, at = @At("HEAD"))
    public void playSound(double x, double y, double z, SoundEvent sound, SoundCategory category, float volume, float pitch, boolean bl, CallbackInfo ci) {
        ci.cancel();
    }

}
