package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.render.*;
import org.joml.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {
    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    private static void drawWithGlobalProgramInternal(BufferBuilder.BuiltBuffer builtBuffer) {
        ByteBuffer buffer = builtBuffer.getVertexBuffer();
        BufferBuilder.DrawParameters parameters = builtBuffer.getParameters();

        VertexFormat vertexFormat = parameters.format();
        if (vertexFormat == VertexFormats.POSITION_COLOR) {
            if (vertexFormat.getElements().get(1).getComponentType() == VertexFormatElement.ComponentType.UBYTE)
                WgpuNative.wmUsePipeline(0);
            else if (vertexFormat.getElements().get(1).getComponentType() == VertexFormatElement.ComponentType.FLOAT)
                WgpuNative.wmUsePipeline(2);
            else return;
        } else if (vertexFormat == VertexFormats.POSITION_TEXTURE) {
            WgpuNative.wmUsePipeline(1);
            WgpuNative.attachTextureBindGroup(0, RenderSystem.getShaderTexture(0));
        } else if (vertexFormat == VertexFormats.POSITION_COLOR_TEXTURE_LIGHT) {
            //Text rendering
            WgpuNative.wmUsePipeline(3);
            WgpuNative.attachTextureBindGroup(0, RenderSystem.getShaderTexture(0));
        } else if (vertexFormat == VertexFormats.POSITION_TEXTURE_COLOR) {
            WgpuNative.wmUsePipeline(4);
            WgpuNative.attachTextureBindGroup(0, RenderSystem.getShaderTexture(0));
        } else {
            return;
        }

        Matrix4f mat = RenderSystem.getProjectionMatrix();
        Matrix4f mat1 = RenderSystem.getModelViewMatrix();

        Matrix4f mat2 = (new Matrix4f()).zero().add(mat).mul(mat1);

        float[] out = new float[16];
        mat2.get(out);
        WgpuNative.setProjectionMatrix(out);

        int count = parameters.vertexCount();
        byte[] bytes = new byte[count * vertexFormat.getVertexSizeByte()];
        buffer.get(bytes);

        WgpuNative.setVertexBuffer(bytes);

        if (parameters.mode() == VertexFormat.DrawMode.QUADS) {
            int[] quadIndices = new int[count * 6];

            for (int i = 0; i < count; i++) {
                quadIndices[(i * 6)] = i * 4;
                quadIndices[(i * 6) + 1] = (i * 4) + 1;
                quadIndices[(i * 6) + 2] = (i * 4) + 3;
                quadIndices[(i * 6) + 3] = (i * 4) + 1;
                quadIndices[(i * 6) + 4] = (i * 4) + 2;
                quadIndices[(i * 6) + 5] = (i * 4) + 3;
            }

            WgpuNative.setIndexBuffer(quadIndices);
            WgpuNative.drawIndexed(count + (count / 2));
        } else if (parameters.mode() == VertexFormat.DrawMode.TRIANGLES) {
            WgpuNative.draw(parameters.vertexCount());
        }

        builtBuffer.release();
    }

}
