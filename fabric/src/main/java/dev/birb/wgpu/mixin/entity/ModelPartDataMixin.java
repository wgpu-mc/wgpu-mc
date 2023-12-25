package dev.birb.wgpu.mixin.entity;

import com.google.common.collect.ImmutableList;
import dev.birb.wgpu.entity.ModelPartNameAccessor;
import it.unimi.dsi.fastutil.objects.Object2ObjectArrayMap;
import net.minecraft.client.model.ModelCuboidData;
import net.minecraft.client.model.ModelPart;
import net.minecraft.client.model.ModelPartData;
import net.minecraft.client.model.ModelTransform;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

@Mixin(ModelPartData.class)
public class ModelPartDataMixin {

    @Shadow @Final private Map<String, ModelPartData> children;

    @Shadow @Final private List<ModelCuboidData> cuboidData;

    @Shadow @Final private ModelTransform rotationData;

    /**
     * @author wgpu-mc
     * @reason Intercept some stuff
     */
    @Overwrite
    public ModelPart createPart(int textureWidth, int textureHeight) {
        Object2ObjectArrayMap<String, ModelPart> object2ObjectArrayMap = (Object2ObjectArrayMap)this.children.entrySet().stream().collect(Collectors.toMap(Map.Entry::getKey, (entry) -> {
            ModelPart part = ((ModelPartData)entry.getValue()).createPart(textureWidth, textureHeight);
            ((ModelPartNameAccessor) (Object) part).setName(entry.getKey());
            return part;
        }, (modelPartx, modelPart2) -> {
            return modelPartx;
        }, Object2ObjectArrayMap::new));
        List<ModelPart.Cuboid> list = (List)this.cuboidData.stream().map((modelCuboidData) -> {
            return modelCuboidData.createCuboid(textureWidth, textureHeight);
        }).collect(ImmutableList.toImmutableList());
        ModelPart modelPart = new ModelPart(list, object2ObjectArrayMap);
        modelPart.setTransform(this.rotationData);
        return modelPart;
    }

}
