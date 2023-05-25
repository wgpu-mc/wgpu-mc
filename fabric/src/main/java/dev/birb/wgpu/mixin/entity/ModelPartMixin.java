package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import dev.birb.wgpu.entity.ModelPartNameAccessor;
import net.minecraft.client.model.ModelPart;
import net.minecraft.client.render.VertexConsumer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.math.Matrix3f;
import net.minecraft.util.math.Matrix4f;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.Inject;

import java.util.Iterator;
import java.util.List;
import java.util.Map;

@Mixin(ModelPart.class)
public abstract class ModelPartMixin implements ModelPartNameAccessor {

    @Shadow public boolean visible;

    @Shadow @Final private List<ModelPart.Cuboid> cuboids;

    @Shadow @Final private Map<String, ModelPart> children;

    @Shadow public abstract void rotate(MatrixStack matrices);

    @Shadow protected abstract void renderCuboids(MatrixStack.Entry entry, VertexConsumer vertexConsumer, int light, int overlay, float red, float green, float blue, float alpha);

    private String name;
    private int partIndex;

    @Override
    public String getName() {
        return name;
    }

    @Override
    public void setName(String name) {
        this.name = name;
    }

    /**
     * @author wgpu-mc
     * @reason Render entities in Rust
     */
    @Overwrite
    public void render(MatrixStack matrices, VertexConsumer vertices, int light, int overlay, float red, float green, float blue, float alpha) {
        if (!this.cuboids.isEmpty() || !this.children.isEmpty()) {
            if(!this.visible) {
                //TODO
            }

            matrices.push();

            String thisPartName = ((ModelPartNameAccessor) (Object) this).getName();

            if(thisPartName == null) {
                thisPartName = "root";
            }

            this.rotate(matrices);
            Matrix4f mat4 = matrices.peek().getPositionMatrix();
            EntityState.entityModelMatrices.put(thisPartName, mat4);

            Matrix3f normalMat3 = matrices.peek().getNormalMatrix();

            Iterator var9 = this.children.values().iterator();

            while(var9.hasNext()) {
                ModelPart modelPart = (ModelPart)var9.next();
                modelPart.render(matrices, vertices, light, overlay, red, green, blue, alpha);
            }

            matrices.pop();
        }
    }

}
