package dev.birb.wgpu.mixin.entity;

import net.minecraft.client.render.entity.EntityRenderDispatcher;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.entity.Entity;
import net.minecraft.entity.EntityType;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import java.util.Map;

@Mixin(EntityRenderDispatcher.class)
public abstract class EntityRenderDispatcherMixin {

    @Shadow public abstract <T extends Entity> EntityRenderer<? super T, ?> getRenderer(T entity);

    @Shadow private Map<EntityType<?>, EntityRenderer<?, ?>> renderers;

    private static int getOverlayColor(int packedUV) {
        int u = packedUV & 0xffff;
        int v = packedUV >> 16;

        if (v < 8) return -1308622593;

        return (((int)((1.0f - (float)u / 15.0f * 0.75f) * 255.0f)) << 24) | 0xFFFFFF;
    }

//    @Inject(method = "render", at = @At("TAIL"))
//    private <S extends EntityRenderState> void render(S state, double x, double y, double z, MatrixStack matrices, VertexConsumerProvider vertexConsumers, int light, EntityRenderer<?, S> renderer) {

//        //TODO implement FallingBlockEntity
//        if(entity instanceof FallingBlockEntity) return;
//
//        if(type == EntityType.ITEM) return;
//
//        EntityState.instanceOverlay = 0xffffffff;
//
//        if(renderer instanceof LivingEntityRenderer<?, ?, ?> livingRenderer) {
//            LivingEntity livingEntity = (LivingEntity) entity;
////            EntityState.instanceOverlay = getOverlayColor(LivingEntityRenderer.getOverlay(livingEntity, livingRenderer.getAnimationCounter(livingEntity, tickDelta)));
//            EntityState.instanceOverlay = getOverlayColor(LivingEntityRenderer.getOverlay(livingEntity, 0.0f));
//        }
//
//        String rootLayerName;
//
//        if(type == EntityType.PLAYER) {
//            rootLayerName = EntityModelLayers.PLAYER.toString();
//        } else {
//            EntityState.EntityModelInfo info = EntityState.layers.get(type);
//            boolean debugBreak = false;
//            if(info == null) return;
//
//            rootLayerName = info.root.toString();
//            if(rootLayerName == null) return;
//        }
//
//        Identifier textureIdentifier = this.getRenderer(entity).getTexture(entity);
//        TextureManager textureManager = MinecraftClient.getInstance().getTextureManager();
//        int glId = textureManager.getTexture(textureIdentifier).getGlId();
//
//        EntityState.assembleEntity(rootLayerName, glId);
//        EntityState.entityModelPartStates.clear();
//    }

}
