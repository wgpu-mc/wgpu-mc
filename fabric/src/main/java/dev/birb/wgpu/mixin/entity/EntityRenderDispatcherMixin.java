package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.render.entity.EntityRenderDispatcher;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.render.entity.model.EntityModelLayers;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import net.minecraft.entity.EntityType;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(EntityRenderDispatcher.class)
public abstract class EntityRenderDispatcherMixin {

    @Shadow public abstract <T extends Entity> EntityRenderer<? super T> getRenderer(T entity);

    @Inject(method = "render", at = @At("TAIL"))
    public<E extends Entity> void render(E entity, double x, double y, double z, float yaw, float tickDelta, MatrixStack matrices, VertexConsumerProvider vertexConsumers, int light, CallbackInfo ci) {
        EntityType<?> type = entity.getType();
        String rootLayerName;

        if(type == EntityType.PLAYER) {
            rootLayerName = EntityModelLayers.PLAYER.getName();
        } else {
            EntityState.EntityModelInfo info = EntityState.layers.get(type);
            rootLayerName = info.root.getName();
        }

        EntityState.assembleEntity(rootLayerName);
    }

}
