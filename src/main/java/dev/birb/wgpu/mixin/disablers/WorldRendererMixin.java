package dev.birb.wgpu.mixin.disablers;

import net.minecraft.block.BlockState;
import net.minecraft.client.particle.Particle;
import net.minecraft.client.render.WorldRenderer;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.particle.ParticleEffect;
import net.minecraft.util.math.BlockPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(WorldRenderer.class)
public class WorldRendererMixin {

    @Inject(method = "scheduleBlockRerenderIfNeeded", at = @At("HEAD"), cancellable = true)
    private void scheduleBlockRerenderIfNeeded(BlockPos pos, BlockState old, BlockState updated, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "spawnParticle(Lnet/minecraft/particle/ParticleEffect;ZZDDDDDD)Lnet/minecraft/client/particle/Particle;", at = @At("HEAD"), cancellable = true)
    private void spawnParticle(ParticleEffect parameters, boolean alwaysSpawn, boolean canSpawnOnMinimal, double x, double y, double z, double velocityX, double velocityY, double velocityZ, CallbackInfoReturnable<Particle> cir) {
        cir.cancel();
    }

    @Inject(method = "scheduleChunkRender", at = @At("HEAD"), cancellable = true)
    private void scheduleChunkRender(int x, int y, int z, boolean important, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "processGlobalEvent", at = @At("HEAD"), cancellable = true)
    private void processGlobalEvent(int eventId, BlockPos pos, int i, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "processWorldEvent", at = @At("HEAD"), cancellable = true)
    private void processWorldEvent(PlayerEntity source, int eventId, BlockPos pos, int data, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderStars()V", at = @At("HEAD"), cancellable = true)
    private void renderStars(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderLightSky()V", at = @At("HEAD"), cancellable = true)
    private void renderLightSky(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderDarkSky()V", at = @At("HEAD"), cancellable = true)
    private void renderDarkSky(CallbackInfo ci) {
        ci.cancel();
    }

}
