package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.LightmapTextureManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(LightmapTextureManager.class)
public class LightmapTextureManagerMixin {
    @Inject(at = @At("TAIL"), method = "<init>")
    void constructor(GameRenderer renderer, MinecraftClient client, CallbackInfo ci) {
        LightmapTextureManager thiz = (LightmapTextureManager) (Object) this;

//        thiz.texture.getGlId();
        WgpuNative.setLightmapID(thiz.texture.getGlId());
    }

}
