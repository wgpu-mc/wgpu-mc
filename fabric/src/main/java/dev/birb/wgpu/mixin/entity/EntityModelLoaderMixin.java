package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import dev.birb.wgpu.entity.ModelPartAccessor;
import dev.birb.wgpu.entity.ModelPartNameAccessor;
import dev.birb.wgpu.render.Wgpu;
import net.minecraft.client.model.ModelPart;
import net.minecraft.client.model.TexturedModelData;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.render.entity.model.EntityModelLoader;
import net.minecraft.resource.ResourceManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.HashMap;
import java.util.Map;

@Mixin(EntityModelLoader.class)
public class EntityModelLoaderMixin {

//    @Shadow private Map<EntityModelLayer, TexturedModelData> modelParts;

    @Shadow private Map<EntityModelLayer, TexturedModelData> modelParts;

    private void recurseModelPartApplyId(HashMap<String, Integer> indices, ModelPart part, String partName) {
        int index = indices.get(partName);
        ((ModelPartAccessor) (Object) part).setModelPartIndex(index);

        part.children.forEach((name, child) -> {
            recurseModelPartApplyId(indices, child, name);
        });
    }

    @Inject(method = "getModelPart", at = @At("RETURN"))
    private void injectPartIds(EntityModelLayer layer, CallbackInfoReturnable<ModelPart> cir) {
        ModelPart modelPart = cir.getReturnValue();
        Map<EntityModelLayer, TexturedModelData> modelParts = this.modelParts;

        Wgpu.injectPartIds.add(() -> {
            TexturedModelData tmd = modelParts.get(layer);
            HashMap<String, Integer> indices = EntityState.matrixIndices.get(layer.toString());
            if(indices == null) return;
            String rootName = ((ModelPartNameAccessor) (Object) modelPart).getName();
            recurseModelPartApplyId(indices, modelPart, "root");
        });
    }

}
