package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.entity.DummyVertexConsumer;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.render.*;
import net.minecraft.client.render.entity.EntityRenderDispatcher;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.entity.Entity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.math.MathHelper;
import net.minecraft.util.math.RotationAxis;
import net.minecraft.util.math.Vec3d;
import org.jetbrains.annotations.Nullable;
import org.joml.Matrix4f;
import org.joml.Vector3d;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.blaze3d.systems.RenderSystem;

import com.mojang.blaze3d.systems.RenderSystem;

import java.util.Objects;

@Mixin(WorldRenderer.class)
public abstract class WorldRendererMixin {

    @Shadow
    public abstract void updateChunks(Camera camera);

    @Shadow @Final private MinecraftClient client;

    @Shadow protected abstract void setupTerrain(Camera camera, Frustum frustum, boolean hasForcedFrustum, boolean spectator);

    @Shadow private Frustum frustum;

    @Shadow private @Nullable Frustum capturedFrustum;

    @Shadow private boolean shouldCaptureFrustum;

    @Shadow @Final private Vector3d capturedFrustumPosition;

    @Shadow private @Nullable ClientWorld world;

    @Shadow @Final private EntityRenderDispatcher entityRenderDispatcher;

    @Shadow protected abstract void renderEntity(Entity entity, double cameraX, double cameraY, double cameraZ, float tickDelta, MatrixStack matrices, VertexConsumerProvider vertexConsumers);

    @Shadow private int ticks;

    @Shadow protected abstract void captureFrustum(Matrix4f positionMatrix, Matrix4f projectionMatrix, double x, double y, double z, Frustum frustum);
    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    private void renderLightSky() {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    private void renderDarkSky() {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    private void renderStars() {

    }

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void reload(ResourceManager manager) {
    }

    @Inject(method = "render", cancellable = true, at = @At("HEAD"))
    public void render(MatrixStack matrices, float tickDelta, long limitTime, boolean renderBlockOutline, Camera camera, GameRenderer gameRenderer, LightmapTextureManager lightmapTextureManager, Matrix4f projectionMatrix, CallbackInfo ci) {
        Vec3d translate = camera.getPos();
        
        BackgroundRenderer.render(camera, tickDelta, this.world, this.client.options.getClampedViewDistance(), gameRenderer.getSkyDarkness(tickDelta));

        Frustum currentFrustum;
        if (this.capturedFrustum != null) {
            currentFrustum = this.capturedFrustum;
            currentFrustum.setPosition(this.capturedFrustumPosition.x, this.capturedFrustumPosition.y, this.capturedFrustumPosition.z);
        } else {
            currentFrustum = this.frustum;
        }

        if (this.shouldCaptureFrustum) {
            captureFrustum(matrices.peek().getPositionMatrix(), projectionMatrix, translate.x, translate.y, translate.z, this.capturedFrustum != null ? new Frustum(matrices.peek().getPositionMatrix(), projectionMatrix) : currentFrustum);
            this.shouldCaptureFrustum = false;
        } 
        
        Objects.requireNonNull(this.world).runQueuedChunkUpdates();
        this.world.getChunkManager().getLightingProvider().doLightUpdates();

        this.setupTerrain(camera, currentFrustum, this.capturedFrustum != null, this.client.player != null && this.client.player.isSpectator());
        this.updateChunks(camera);

        bindSkyData(matrices, projectionMatrix, tickDelta, camera);
        // -- Camera --

        MatrixStack cameraStack = new MatrixStack();
        cameraStack.loadIdentity();

        ClientPlayerEntity player = MinecraftClient.getInstance().player;

        // -- Entities --

//        this.blockEntityRenderDispatcher.configure(this.world, camera, this.client.crosshairTarget);
        this.entityRenderDispatcher.configure(this.world, camera, this.client.targetedEntity);

        if(this.world != null) {
            MatrixStack entityStack = new MatrixStack();
            entityStack.loadIdentity();
            VertexConsumerProvider dummyProvider = layer -> new DummyVertexConsumer();

            for(Entity entity : this.world.getEntities()) {
                if((entity != camera.getFocusedEntity() || camera.isThirdPerson() || camera.getFocusedEntity() instanceof LivingEntity && ((LivingEntity)camera.getFocusedEntity()).isSleeping()) && (!(entity instanceof ClientPlayerEntity) || camera.getFocusedEntity() == entity)) {
                    this.renderEntity(entity, 0.0, 64.0, 0.0, tickDelta, entityStack, dummyProvider);
                }
            }
        }

        
        // Update matrices to shader
        matrices.push();
        float[] floatBuffer = new float[16];
        matrices.peek().getPositionMatrix().get(floatBuffer);
        WgpuNative.setMatrix(1, floatBuffer); // Model transformation for shaders
        matrices.pop();

        floatBuffer = new float[16];
        RenderSystem.getModelViewMatrix().get(floatBuffer);
        WgpuNative.setMatrix(2, floatBuffer); // View?

        floatBuffer = new float[16];
        RenderSystem.getProjectionMatrix().get(floatBuffer);
        WgpuNative.setMatrix(0, floatBuffer); // Actual projection

            

        if(player != null) {
            matrices.multiplyPositionMatrix(new Matrix4f().translation(
                    (float) -translate.x,
                    (float) -translate.y - 64.0f,
                    (float) -translate.z
            ));

            floatBuffer = new float[16];
            matrices.peek().getPositionMatrix().get(floatBuffer);
            WgpuNative.setMatrix(3, floatBuffer); // Terrain transformation matrix
        }

        ci.cancel();
    }

    @Inject(method = "setWorld", at = @At("HEAD"))
    public void setWorld(ClientWorld world, CallbackInfo ci) {
        WgpuNative.clearChunks();
    }

    public void bindSkyData(MatrixStack matrices, Matrix4f projectionMatrix, float tickDelta, Camera camera) {
        Vec3d skyColor = this.world.getSkyColor(camera.getPos(), tickDelta);
        float skyAngle = this.world.getSkyAngle(tickDelta);
        float skyBrightness = this.world.getSkyBrightness(tickDelta);
        
        //matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(-90.0F));
       // matrices.multiply(RotationAxis.POSITIVE_X.rotationDegrees(this.world.getSkyAngle(tickDelta) * 360.0F));

        WgpuNative.bindSkyData((float) skyColor.getX(), (float) skyColor.getY(), (float) skyColor.getZ(), skyAngle, skyBrightness, this.world.getMoonPhase());
    }

}
