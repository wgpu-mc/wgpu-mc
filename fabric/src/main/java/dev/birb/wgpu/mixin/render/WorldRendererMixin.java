package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.rust.WgpuNative;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.render.BuiltChunkStorage;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.Frustum;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.WorldRenderer;
import net.minecraft.client.render.chunk.ChunkBuilder;
import net.minecraft.client.render.chunk.ChunkRendererRegionBuilder;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.client.util.math.Vector3d;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.math.Matrix4f;
import net.minecraft.util.math.Vec3d;
import net.minecraft.util.math.Vec3f;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.FloatBuffer;

@Mixin(WorldRenderer.class)
public abstract class WorldRendererMixin {

    @Shadow protected abstract void updateChunks(Camera camera);

    @Shadow @Final private ObjectArrayList<WorldRenderer.ChunkInfo> chunkInfos;

    @Shadow private @Nullable BuiltChunkStorage chunks;

    @Shadow @Final private MinecraftClient client;

    @Shadow protected abstract void setupTerrain(Camera camera, Frustum frustum, boolean hasForcedFrustum, boolean spectator);

    @Shadow private Frustum frustum;

    @Shadow private @Nullable Frustum capturedFrustum;

    @Shadow @Final private Vector3d capturedFrustumPosition;

    @Shadow private @Nullable ClientWorld world;

    /**
     * @author
     */
    @Overwrite
    public void renderLightSky() {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void renderDarkSky() {

    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void renderStars() {

    }

    /**
     * @author wgpu-mc
     * @reason do no such thing
     */
    @Overwrite
    public void reload(ResourceManager manager) {
    }

    @Inject(method = "render", cancellable = true, at = @At("HEAD"))
    public void render(MatrixStack matrices, float tickDelta, long limitTime, boolean renderBlockOutline, Camera camera, GameRenderer gameRenderer, LightmapTextureManager lightmapTextureManager, Matrix4f positionMatrix, CallbackInfo ci) {
        ChunkRendererRegionBuilder builder = new ChunkRendererRegionBuilder();

        Frustum frustum;
        if (this.capturedFrustum != null) {
            frustum = this.capturedFrustum;
            frustum.setPosition(this.capturedFrustumPosition.x, this.capturedFrustumPosition.y, this.capturedFrustumPosition.z);
        } else {
            frustum = this.frustum;
        }

        this.world.runQueuedChunkUpdates();
        this.setupTerrain(camera, frustum, this.capturedFrustum != null, this.client.player.isSpectator());
        this.updateChunks(camera);

        MatrixStack stack = new MatrixStack();
        stack.loadIdentity();

        ClientPlayerEntity player = MinecraftClient.getInstance().player;

        if(player != null) {
            ChunkPos pos = player.getChunkPos();

            Vec3d translate = camera.getPos();

            stack.peek().getPositionMatrix().multiply(Vec3f.POSITIVE_X.getDegreesQuaternion(camera.getPitch()));
            stack.peek().getPositionMatrix().multiply(Vec3f.POSITIVE_Y.getDegreesQuaternion(camera.getYaw() + 180.0f));

            //Java does negative modulo in an annoying way, e.g. -1 % 16.0 = -1.0 and not 15.0
            double modX = ((translate.x % 16.0) + 16.0) % 16.0;
            double modZ = ((translate.z % 16.0) + 16.0) % 16.0;

//            stack.peek().getPositionMatrix().multiply(Matrix4f.translate(
//                    (float) (-modX),
//                    ((float) -translate.y) - 64.0f,
//                    (float) (-modZ)
//            ));

            stack.peek().getPositionMatrix().multiply(Matrix4f.translate(
                    (float) -translate.x,
                    (float) -translate.y - 64.0f,
                    (float) -translate.z
            ));

//            WgpuNative.setChunkOffset(-(int) (Math.floor(translate.x / 16.0)), -(int) (Math.floor(translate.z / 16.0)));
        }

        FloatBuffer floatBuffer = FloatBuffer.allocate(16);
        float[] out = new float[16];
        stack.peek().getPositionMatrix().writeColumnMajor(floatBuffer);
        floatBuffer.get(out);

        WgpuNative.setMatrix(0, out);

        ci.cancel();
    }

    @Inject(method = "setWorld", at = @At("HEAD"))
    public void setWorld(ClientWorld world, CallbackInfo ci) {
        WgpuNative.clearChunks();
    }

}
