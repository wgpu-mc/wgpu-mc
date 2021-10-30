package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.texture.AbstractTexture;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(AbstractTexture.class)
public class AbstractTextureMixin {

    @Inject(method = "getGlId", at = @At("HEAD"), cancellable = true)
    private void getGlId(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(0);
    }

}
