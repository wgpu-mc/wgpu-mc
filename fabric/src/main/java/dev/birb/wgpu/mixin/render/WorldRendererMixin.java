package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.entity.DummyVertexConsumer;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.render.*;
import net.minecraft.client.render.block.entity.BlockEntityRenderDispatcher;
import net.minecraft.client.render.entity.EntityRenderDispatcher;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.entity.Entity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.math.RotationAxis;
import net.minecraft.util.math.Vec3d;
import org.jetbrains.annotations.Nullable;
import org.joml.Matrix4f;
import org.joml.Vector3d;
import org.lwjgl.BufferUtils;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.FloatBuffer;
import java.util.Objects;

@Mixin(WorldRenderer.class)
public abstract class WorldRendererMixin {

    @Shadow
    public abstract void updateChunks(Camera camera);

    @Shadow @Final private MinecraftClient client;

    @Shadow protected abstract void setupTerrain(Camera camera, Frustum frustum, boolean hasForcedFrustum, boolean spectator);

    @Shadow private Frustum frustum;

    @Shadow private @Nullable Frustum capturedFrustum;

    @Shadow @Final private Vector3d capturedFrustumPosition;

    @Shadow private @Nullable ClientWorld world;

    @Shadow @Final private EntityRenderDispatcher entityRenderDispatcher;

    @Shadow protected abstract void renderEntity(Entity entity, double cameraX, double cameraY, double cameraZ, float tickDelta, MatrixStack matrices, VertexConsumerProvider vertexConsumers);

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
        Frustum currentFrustum;
        if (this.capturedFrustum != null) {
            currentFrustum = this.capturedFrustum;
            currentFrustum.setPosition(this.capturedFrustumPosition.x, this.capturedFrustumPosition.y, this.capturedFrustumPosition.z);
        } else {
            currentFrustum = this.frustum;
        }

        Objects.requireNonNull(this.world).runQueuedChunkUpdates();
        this.world.getChunkManager().getLightingProvider().doLightUpdates();

        this.setupTerrain(camera, currentFrustum, this.capturedFrustum != null, this.client.player != null && this.client.player.isSpectator());
        this.updateChunks(camera);

        // -- Camera --

        MatrixStack cameraStack = new MatrixStack();
        cameraStack.loadIdentity();

        ClientPlayerEntity player = MinecraftClient.getInstance().player;

        if(player != null) {
            Vec3d translate = camera.getPos();

            cameraStack.multiplyPositionMatrix(new Matrix4f().rotation(RotationAxis.POSITIVE_X.rotationDegrees(camera.getPitch())));
            cameraStack.multiplyPositionMatrix(new Matrix4f().rotation(RotationAxis.POSITIVE_Y.rotationDegrees(camera.getYaw() + 180.0f)));

            cameraStack.multiplyPositionMatrix(new Matrix4f().translation(
                    (float) -translate.x,
                    (float) -translate.y - 64.0f,
                    (float) -translate.z
            ));
        }

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

        float[] floatBuffer = new float[16];

        cameraStack.peek().getPositionMatrix().get(floatBuffer);

        WgpuNative.setMatrix(0, floatBuffer);

        ci.cancel();
    }

    @Inject(method = "setWorld", at = @At("HEAD"))
    public void setWorld(ClientWorld world, CallbackInfo ci) {
        WgpuNative.clearChunks();
    }

}
