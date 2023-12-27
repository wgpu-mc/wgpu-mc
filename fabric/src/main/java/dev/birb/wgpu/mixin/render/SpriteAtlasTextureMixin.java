package dev.birb.wgpu.mixin.render;

import net.minecraft.client.texture.SpriteAtlasTexture;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(SpriteAtlasTexture.class)
public class SpriteAtlasTextureMixin {

    @Inject(method = "tickAnimatedSprites", cancellable = true, at = @At("HEAD"))
    public void dontTickAnimatedSprites(CallbackInfo ci) {
        ci.cancel();
    }

}
