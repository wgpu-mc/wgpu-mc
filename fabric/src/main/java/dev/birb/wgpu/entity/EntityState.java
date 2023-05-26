package dev.birb.wgpu.entity;

import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.EntityType;
import net.minecraft.util.math.Matrix4f;

import java.nio.BufferOverflowException;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;
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

    public static class ModelPartState {
        public Matrix4f mat;
        public int overlay;
    }

//    public static List<MatrixIndexTuple> entityModelMatrices = new ArrayList<>();
    public static HashMap<String, ModelPartState> entityModelPartStates = new HashMap<>();
    public static int instanceOverlay = 0xffffffff;

    public static final HashMap<String, EntityRenderState> renderStates = new HashMap<>();
    public static final HashMap<String, HashMap<String, Integer>> matrixIndices = new HashMap<>();

    public static void assembleEntity(String entityName, int textureId) {
        HashMap<String, Integer> partIndices = matrixIndices.get(entityName);
        Matrix4f[] orderedMatrices = new Matrix4f[partIndices.size()];
        int[] overlays = new int[partIndices.size()];

        for(Map.Entry<String, ModelPartState> entry : entityModelPartStates.entrySet()) {
//        for(Matrix4f mat : entityModelMatrices) {
            String partName = entry.getKey();
            Matrix4f mat = entry.getValue().mat;

            if(!partIndices.containsKey(partName)) return;

            int partIndex = partIndices.get(partName);
            orderedMatrices[partIndex] = mat;
            overlays[partIndex] = entry.getValue().overlay;
        }

        EntityRenderState state = renderStates.getOrDefault(entityName, new EntityRenderState());
        state.overlays.put(overlays);

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

            try {
                state.buffer.put(floatBufTemp);
            } catch(BufferOverflowException e) {
                FloatBuffer oldBuffer = state.buffer;
                state.buffer = FloatBuffer.allocate(state.buffer.capacity() + 10000);
                state.buffer.put(oldBuffer);
//                state.buffer.position()
            }
            floatBufTemp.position(0);
        }

        state.textureId = textureId;
        state.count++;

        renderStates.put(entityName, state);
    }

    public static class EntityRenderState {

        public FloatBuffer buffer = FloatBuffer.allocate(100000);
        public final IntBuffer overlays = IntBuffer.allocate(100000);
        public int count = 0;
        public int textureId;

    }

    public static class EntityModelInfo {

        public EntityModelLayer root;
        public final List<EntityModelLayer> features = new ArrayList<>();

    }

}
