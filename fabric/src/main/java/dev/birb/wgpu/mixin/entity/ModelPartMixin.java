package dev.birb.wgpu.mixin.entity;

import dev.birb.wgpu.entity.EntityState;
import dev.birb.wgpu.entity.ModelPartAccessor;
import dev.birb.wgpu.entity.ModelPartNameAccessor;
import net.minecraft.client.model.ModelPart;
import net.minecraft.client.render.VertexConsumer;
import net.minecraft.client.util.math.MatrixStack;
import org.joml.Matrix3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.List;
import java.util.Map;

@Mixin(ModelPart.class)
public abstract class ModelPartMixin implements ModelPartNameAccessor, ModelPartAccessor {

    @Shadow public boolean visible;

    @Shadow protected abstract void renderCuboids(MatrixStack.Entry entry, VertexConsumer vertexConsumer, int light, int overlay, int color);

    @Shadow @Final private List<ModelPart.Cuboid> cuboids;
    @Shadow @Final public Map<String, ModelPart> children;

    @Shadow public abstract void rotate(Vector3f vec3f);

    @Shadow public abstract void applyTransform(MatrixStack matrices);

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
    public void render(MatrixStack matrices, VertexConsumer vertices, int light, int overlay, int color) {
        if (!this.cuboids.isEmpty() || !this.children.isEmpty()) {
            int actualOverlay = EntityState.instanceOverlay;

            //sets the alpha to 0
            if(!this.visible) actualOverlay = 0;

            matrices.push();

            this.applyTransform(matrices);
            Matrix4f mat4 = matrices.peek().getPositionMatrix();

            String thisPartName = ((ModelPartNameAccessor) (Object) this).getName();

            if(thisPartName == null) {
                thisPartName = "root";
            }

            EntityState.ModelPartState state = new EntityState.ModelPartState();
            state.overlay = actualOverlay;
            state.mat = mat4;

            EntityState.entityModelPartStates.put(thisPartName, state);

            Matrix3f normalMat3 = matrices.peek().getNormalMatrix();

            for (ModelPart modelPart : this.children.values()) {
                modelPart.render(matrices, vertices, light, overlay, color);
            }

            matrices.pop();
        }
    }

    @Override
    public void setModelPartIndex(int partIndex) {
        this.partIndex = partIndex;
    }

}
