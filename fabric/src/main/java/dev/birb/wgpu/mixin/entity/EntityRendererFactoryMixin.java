package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import net.minecraft.client.model.ModelPart;
import net.minecraft.client.render.entity.EntityRendererFactory;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(EntityRendererFactory.Context.class)
public class EntityRendererFactoryMixin {

    @Inject(method = "getPart", at = @At("HEAD"))
    public void getPart(EntityModelLayer layer, CallbackInfoReturnable<ModelPart> cir) {
        if(EntityState.registeringRoot) {
            EntityState.EntityModelInfo info = new EntityState.EntityModelInfo();
            info.root = layer;
            EntityState.layers.put(EntityState.builderType, info);
            EntityState.registeringRoot = false;
        } else {
            EntityState.EntityModelInfo info = EntityState.layers.get((EntityState.builderType));
            info.features.add(layer);
        }
    }

}
