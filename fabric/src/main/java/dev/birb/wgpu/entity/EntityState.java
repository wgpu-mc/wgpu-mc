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

//    public static HashMap<String, ModelPartState> entityModelPartStates = new HashMap<>();
    public static ModelPartState[] entityModelPartStates = new ModelPartState[1000];

    public static int instanceOverlay = 0xffffffff;

    public static final HashMap<String, EntityRenderState> renderStates = new HashMap<>();
    public static final HashMap<String, HashMap<String, Integer>> matrixIndices = new HashMap<>();

    public static void assembleEntity(String entityName, int textureId) {
        HashMap<String, Integer> partIndices = matrixIndices.get(entityName);

        MatrixStack stack = new MatrixStack();
        stack.loadIdentity();

        EntityRenderState state = renderStates.getOrDefault(entityName, new EntityRenderState());

        FloatBuffer floatBufTemp = FloatBuffer.allocate(16);

        for(int i=0;i<partIndices.size();i++) {
            ModelPartState modelPartState = entityModelPartStates[i];

            if(modelPartState == null) modelPartState = new ModelPartState();

            Matrix4f mat = modelPartState.mat;
            state.overlays.put(modelPartState.overlay);

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
