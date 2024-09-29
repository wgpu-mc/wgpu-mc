package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.resource.ResourceFactory;
import net.minecraft.resource.ResourceManager;
import net.minecraft.resource.ResourceReloader;
import net.minecraft.resource.SinglePreparationResourceReloader;
import net.minecraft.util.profiler.Profiler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GameRenderer.class)
public abstract class GameRendererMixin {

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void preloadPrograms(ResourceFactory factory) {

    }

    @Inject(at = @At("HEAD"), method = "renderHand", cancellable = true)
    public void renderHand(MatrixStack matrices, Camera camera, float tickDelta, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(at = @At("RETURN"), method = "render")
    public void render(float tickDelta, long startTime, boolean tick, CallbackInfo ci) {
        WgpuNative.render(tickDelta,startTime,tick);
    }


    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public ResourceReloader createProgramReloader() {
        // created just not to make it null, I wouldn't want minecraft to explode because of this
        return new SinglePreparationResourceReloader<>() {
            @Override
            protected Object prepare(ResourceManager manager, Profiler profiler) {
                return null;
            }

            @Override
            protected void apply(Object prepared, ResourceManager manager, Profiler profiler) {

            }
        };
    }
}
