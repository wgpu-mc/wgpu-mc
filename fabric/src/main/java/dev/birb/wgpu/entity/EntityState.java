package dev.birb.wgpu.entity;

import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.EntityType;
import net.minecraft.util.math.Matrix4f;

import java.nio.BufferOverflowException;
import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;

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
            state.overlayView.put(modelPartState.overlay);

            if (mat == null) {
                mat = stack.peek().getPositionMatrix();
            }
            mat.writeColumnMajor(floatBufTemp);

            try {
                state.matView.put(floatBufTemp);
            } catch(BufferOverflowException e) {
                int oldPosition = state.matView.position();

                ByteBuffer oldBuffer = state.matBuffer;
                state.matBuffer = ByteBuffer.allocateDirect(oldBuffer.capacity() + 40000);
                state.matBuffer.put(oldBuffer);
                state.matBuffer.position(0);

                state.matView = state.matBuffer.asFloatBuffer();
                state.matView.position(oldPosition);

                state.matView.put(floatBufTemp);
            }

            floatBufTemp.position(0);
        }

        state.textureId = textureId;
        state.count++;

        renderStates.put(entityName, state);
    }

    public static class EntityRenderState {

        public ByteBuffer matBuffer = ByteBuffer.allocateDirect(100000 * 4);
        public FloatBuffer matView = matBuffer.asFloatBuffer();

        public final ByteBuffer overlays = ByteBuffer.allocateDirect(100000 * 4);
        public final IntBuffer overlayView = overlays.asIntBuffer();

        public int count = 0;
        public int textureId;

        public void clear() {
            this.matView.clear();
            this.overlayView.clear();

            this.count = 0;
        }

    }

    public static class EntityModelInfo {

        public EntityModelLayer root;
        public final List<EntityModelLayer> features = new ArrayList<>();

    }

}
