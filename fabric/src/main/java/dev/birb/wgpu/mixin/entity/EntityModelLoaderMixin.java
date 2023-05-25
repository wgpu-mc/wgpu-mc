package dev.birb.wgpu.mixin.entity;

import net.minecraft.client.model.TexturedModelData;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.render.entity.model.EntityModelLoader;
import net.minecraft.resource.ResourceManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

@Mixin(EntityModelLoader.class)
public class EntityModelLoaderMixin {

    @Shadow private Map<EntityModelLayer, TexturedModelData> modelParts;

    @Inject(method = "reload", at = @At("TAIL"))
    private void setModelPartIndices(ResourceManager manager, CallbackInfo ci) {
//        for(Map.Entry<EntityModelLayer, TexturedModelData> entry : this.modelParts.entrySet()) {
//            entry.getValue().createModel();
//        }
    }

}
