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

    @Redirect(method = "renderWorld", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/render/WorldRenderer;render(Lnet/minecraft/client/util/math/MatrixStack;FJZLnet/minecraft/client/render/Camera;Lnet/minecraft/client/render/GameRenderer;Lnet/minecraft/client/render/LightmapTextureManager;Lnet/minecraft/util/math/Matrix4f;)V"))
    public void redirectRenderWorld(WorldRenderer instance, MatrixStack matrices, float tickDelta, long limitTime, boolean renderBlockOutline, Camera camera, GameRenderer gameRenderer, LightmapTextureManager lightmapTextureManager, Matrix4f positionMatrix) {
//        Matrix4f mat = matrices.peek().getPositionMatrix();
        MatrixStack stack = new MatrixStack();
        stack.loadIdentity();

        ClientPlayerEntity player = MinecraftClient.getInstance().player;

        if(player != null) {
            ChunkPos pos = player.getChunkPos();
            WgpuNative.setChunkOffset(pos.x, pos.z);
            Vec3d translate = camera.getPos().multiply(-1.0);

            stack.peek().getPositionMatrix().multiply(Vec3f.POSITIVE_X.getDegreesQuaternion(camera.getPitch()));
            stack.peek().getPositionMatrix().multiply(Vec3f.POSITIVE_Y.getDegreesQuaternion(camera.getYaw() + 180.0f));

            stack.peek().getPositionMatrix().multiply(Matrix4f.translate(
                (float) (translate.x),
                (float) translate.y - 64.0f,
                (float) (translate.z)
            ));
        }

        FloatBuffer floatBuffer = FloatBuffer.allocate(16);
        float[] out = new float[16];
        stack.peek().getPositionMatrix().writeColumnMajor(floatBuffer);
        floatBuffer.get(out);
        WgpuNative.setMatrix(0, out);

    }

}
