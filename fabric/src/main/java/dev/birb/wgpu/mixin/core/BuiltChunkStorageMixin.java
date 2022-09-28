package dev.birb.wgpu.mixin.core;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.client.render.BuiltChunkStorage;

@Mixin(BuiltChunkStorage.class)
public class BuiltChunkStorageMixin {

    @Inject(method = "scheduleRebuild(IIIZ)V", at = @At("HEAD"), cancellable = true)
    public void scheduleRerender(int x, int y, int z, boolean important, CallbackInfo callback) {
        callback.cancel();
    }
    
}
