package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.render.entity.EntityRenderDispatcher;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.render.entity.model.EntityModelLayers;
import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.client.texture.TextureManager;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.FallingBlockEntity;
import net.minecraft.util.Identifier;
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

        //TODO implement FallingBlockEntity
        if(entity instanceof FallingBlockEntity) return;

        if(type == EntityType.ITEM) return;

        String rootLayerName;

        if(type == EntityType.PLAYER) {
            rootLayerName = EntityModelLayers.PLAYER.toString();
        } else {
            EntityState.EntityModelInfo info = EntityState.layers.get(type);
            boolean debugBreak = false;
            while(info == null && !debugBreak) {}
            rootLayerName = info.root.toString();
            if(rootLayerName == null) return;
        }

        Identifier textureIdentifier = this.getRenderer(entity).getTexture(entity);
        TextureManager textureManager = MinecraftClient.getInstance().getTextureManager();
        int glId = textureManager.getTexture(textureIdentifier).getGlId();

        EntityState.assembleEntity(rootLayerName, glId);
        EntityState.entityModelMatrices.clear();
    }

}
