package dev.birb.wgpu.entity;

import it.unimi.dsi.fastutil.Hash;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.EntityType;
import net.minecraft.util.math.Matrix4f;

import java.nio.FloatBuffer;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class EntityState {

    public static EntityType<?> builderType;
    public static final HashMap<EntityType<?>, EntityModelInfo> layers = new HashMap<>();
    public static boolean registeringRoot = false;

    public static HashMap<String, Matrix4f> entityModelMatrices = new HashMap<>();

    public static final HashMap<String, EntityRenderState> renderStates = new HashMap<>();
    public static final HashMap<String, HashMap<String, Integer>> matrixIndices = new HashMap<>();

    public static void assembleEntity(String entityName) {
        HashMap<String, Integer> partIndices = matrixIndices.get(entityName);
        Matrix4f[] orderedMatrices = new Matrix4f[partIndices.size()];
        for(Map.Entry<String, Matrix4f> entry : entityModelMatrices.entrySet()) {
            String partName = entry.getKey();
            Matrix4f mat = entry.getValue();

            if(!partIndices.containsKey(partName)) return;

            int partIndex = partIndices.get(partName);
            orderedMatrices[partIndex] = mat;
        }

        EntityRenderState state = renderStates.getOrDefault(entityName, new EntityRenderState());

        MatrixStack stack = new MatrixStack();
        stack.loadIdentity();

//        orderedMatrices[0] = stack.peek().getPositionMatrix();

        for(int i=0;i<orderedMatrices.length;i++) {
            Matrix4f mat = orderedMatrices[i];
            if(mat == null) {
                mat = stack.peek().getPositionMatrix();
            }
            mat.writeColumnMajor(state.buffer);
        }

        state.count++;

        renderStates.put(entityName, state);
    }

    public static class EntityRenderState {

        public final FloatBuffer buffer = FloatBuffer.allocate(50000);
        public int count = 0;

    }

    public static class EntityModelInfo {

        public EntityModelLayer root;
        public final List<EntityModelLayer> features = new ArrayList<>();

    }

}
