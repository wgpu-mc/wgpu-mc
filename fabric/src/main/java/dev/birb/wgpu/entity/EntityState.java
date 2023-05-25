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

    static class MatrixIndexTuple {

        int index;
        Matrix4f mat;

    }

    public static List<MatrixIndexTuple> entityModelMatrices = new ArrayList<>();
//    public static HashMap<String, Matrix4f> entityModelMatrices = new HashMap<>();

    public static final HashMap<String, EntityRenderState> renderStates = new HashMap<>();
    public static final HashMap<String, HashMap<String, Integer>> matrixIndices = new HashMap<>();

    public static void assembleEntity(String entityName, int textureId) {
        HashMap<String, Integer> partIndices = matrixIndices.get(entityName);
        Matrix4f[] orderedMatrices = new Matrix4f[partIndices.size()];
//        for(Map.Entry<String, Matrix4f> entry : entityModelMatrices.entrySet()) {
        for(Matrix4f mat : entityModelMatrices) {
//            String partName = entry.getKey();
//            Matrix4f mat = entry.getValue();

//            if(!partIndices.containsKey(partName)) return;

//            int partIndex = partIndices.get(partName);
            orderedMatrices[partIndex] = mat;
        }

        EntityRenderState state = renderStates.getOrDefault(entityName, new EntityRenderState());

        MatrixStack stack = new MatrixStack();
        stack.loadIdentity();

        FloatBuffer floatBufTemp = FloatBuffer.allocate(16);

//        orderedMatrices[0] = stack.peek().getPositionMatrix();

        for (Matrix4f orderedMatrix : orderedMatrices) {
            Matrix4f mat = orderedMatrix;
            if (mat == null) {
                mat = stack.peek().getPositionMatrix();
            }
            mat.writeColumnMajor(floatBufTemp);

            state.buffer.put(floatBufTemp);
            floatBufTemp.position(0);
        }

        state.textureId = textureId;
        state.count++;

        renderStates.put(entityName, state);
    }

    public static class EntityRenderState {

        public final FloatBuffer buffer = FloatBuffer.allocate(100000);
        public int count = 0;
        public int textureId;

    }

    public static class EntityModelInfo {

        public EntityModelLayer root;
        public final List<EntityModelLayer> features = new ArrayList<>();

    }

}
