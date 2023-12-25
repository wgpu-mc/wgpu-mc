package dev.birb.wgpu.mixin.entity;

import com.google.common.collect.ImmutableMap;
import dev.birb.wgpu.entity.EntityState;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.client.render.entity.EntityRendererFactory;
import net.minecraft.client.render.entity.EntityRenderers;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.player.PlayerEntity;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.Map;

@Mixin(EntityRenderers.class)
public class EntityRenderersMixin {

    @Shadow @Final private static Map<EntityType<?>, EntityRendererFactory<?>> RENDERER_FACTORIES;

    @Shadow @Final private static Map<String, EntityRendererFactory<AbstractClientPlayerEntity>> PLAYER_RENDERER_FACTORIES;

    /**
     * @author wgpu-mc
     * @reason we need to mixin into a lambda
     */
    @Overwrite
    public static Map<EntityType<?>, EntityRenderer<?>> reloadEntityRenderers(EntityRendererFactory.Context ctx) {
        ImmutableMap.Builder<EntityType<?>, EntityRenderer<?>> builder = ImmutableMap.builder();
        RENDERER_FACTORIES.forEach((entityType, factory) -> {
            try {
                EntityState.builderType = entityType;
                EntityState.registeringRoot = true;
                builder.put(entityType, factory.create(ctx));
            } catch (Exception var5) {
                throw new IllegalArgumentException("Failed to create model for  ");
            }
        });
        return builder.build();
    }

    /**
     * @author wgpu-mc
     * @reason we need to mixin into a lambda
     */
    @Overwrite
    public static Map<String, EntityRenderer<? extends PlayerEntity>> reloadPlayerRenderers(EntityRendererFactory.Context ctx) {
        ImmutableMap.Builder<String, EntityRenderer<? extends PlayerEntity>> builder = ImmutableMap.builder();
        PLAYER_RENDERER_FACTORIES.forEach((type, factory) -> {
            try {
                EntityState.builderType = EntityType.PLAYER;
                EntityState.registeringRoot = true;
                builder.put(type, factory.create(ctx));
            } catch (Exception var5) {
                throw new IllegalArgumentException("Failed to create player model for " + type, var5);
            }
        });
        return builder.build();
    }

}
