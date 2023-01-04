package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.WorldRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.resource.ResourceFactory;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.math.Matrix4f;
import net.minecraft.util.math.Vec3d;
import net.minecraft.util.math.Vec3f;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.nio.FloatBuffer;

@Mixin(GameRenderer.class)
public abstract class GameRendererCameraMixin {

    @Shadow @Final private Camera camera;

    @Shadow protected abstract double getFov(Camera camera, float tickDelta, boolean changingFov);

    @Shadow @Final private MinecraftClient client;

    @Shadow public abstract Matrix4f getBasicProjectionMatrix(double fov);

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void preloadShaders(ResourceFactory factory) {

    }

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void reload(ResourceManager manager) {
    }

}
