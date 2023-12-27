package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.OverlayTexture;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.render.entity.EntityRenderDispatcher;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.client.render.entity.LivingEntityRenderer;
import net.minecraft.client.render.entity.model.EntityModelLayers;
import net.minecraft.client.texture.TextureManager;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.FallingBlockEntity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

@Mixin(EntityRenderDispatcher.class)
public abstract class EntityRenderDispatcherMixin {

    @Shadow public abstract <T extends Entity> EntityRenderer<? super T> getRenderer(T entity);

    @Shadow private Map<EntityType<?>, EntityRenderer<?>> renderers;

    private static int getOverlayColor(int packedUV) {
        int u = packedUV & 0xffff;
        int v = packedUV >> 16;

        if (v < 8) return -1308622593;

        return (((int)((1.0f - (float)u / 15.0f * 0.75f) * 255.0f)) << 24) | 0xFFFFFF;
    }

    @Inject(method = "render", at = @At("TAIL"))
    public<E extends Entity> void render(E entity, double x, double y, double z, float yaw, float tickDelta, MatrixStack matrices, VertexConsumerProvider vertexConsumers, int light, CallbackInfo ci) {
        EntityType<?> type = entity.getType();
        EntityRenderer<?> renderer = this.getRenderer(entity);

        //TODO implement FallingBlockEntity
        if(entity instanceof FallingBlockEntity) return;

        if(type == EntityType.ITEM) return;

        EntityState.instanceOverlay = 0xffffffff;

        if(renderer instanceof LivingEntityRenderer livingRenderer) {
            LivingEntity livingEntity = (LivingEntity) entity;
            EntityState.instanceOverlay = getOverlayColor(LivingEntityRenderer.getOverlay(livingEntity, livingRenderer.getAnimationCounter(livingEntity, tickDelta)));
        }

        String rootLayerName;

        if(type == EntityType.PLAYER) {
            rootLayerName = EntityModelLayers.PLAYER.toString();
        } else {
            EntityState.EntityModelInfo info = EntityState.layers.get(type);
            boolean debugBreak = false;
            if(info == null) return;

            rootLayerName = info.root.toString();
            if(rootLayerName == null) return;
        }

        Identifier textureIdentifier = this.getRenderer(entity).getTexture(entity);
        TextureManager textureManager = MinecraftClient.getInstance().getTextureManager();
        int glId = textureManager.getTexture(textureIdentifier).getGlId();

        EntityState.assembleEntity(rootLayerName, glId);
        EntityState.entityModelPartStates.clear();
    }

}
