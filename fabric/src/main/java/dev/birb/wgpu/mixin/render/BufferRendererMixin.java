package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.render.GlWmState;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.Shader;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import net.minecraft.client.render.VertexFormats;
import net.minecraft.client.util.Window;
import net.minecraft.util.math.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.nio.ByteBuffer;
import java.nio.FloatBuffer;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {

    @Shadow private static int currentElementBuffer;

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static void draw(ByteBuffer buffer, VertexFormat.DrawMode drawMode, VertexFormat vertexFormat, int count, VertexFormat.IntType elementFormat, int vertexCount, boolean textured) {
        if(vertexFormat == VertexFormats.POSITION_COLOR) {
            if(vertexFormat.getElements().get(1).getDataType() == VertexFormatElement.DataType.UBYTE)
                WgpuNative.wmUsePipeline(0);
            else if(vertexFormat.getElements().get(1).getDataType() == VertexFormatElement.DataType.FLOAT)
                WgpuNative.wmUsePipeline(2);
            else return;
        } else if(vertexFormat == VertexFormats.POSITION_TEXTURE) {
            WgpuNative.wmUsePipeline(1);
            WgpuNative.attachTextureBindGroup(RenderSystem.getShaderTexture(0));
        } else if(vertexFormat == VertexFormats.POSITION_COLOR_TEXTURE_LIGHT) {
            //Text rendering
            WgpuNative.wmUsePipeline(3);
            WgpuNative.attachTextureBindGroup(RenderSystem.getShaderTexture(0));
        } else {
            return;
        }

        Matrix4f mat = RenderSystem.getProjectionMatrix();
        Matrix4f mat1 = RenderSystem.getModelViewMatrix();

        mat.multiply(mat1);

        FloatBuffer floatBuffer = FloatBuffer.allocate(16);
        float[] out = new float[16];
        mat.writeColumnMajor(floatBuffer);
        floatBuffer.get(out);
        WgpuNative.setProjectionMatrix(out);

        byte[] bytes = new byte[buffer.limit()];
        buffer.get(bytes);

        WgpuNative.setVertexBuffer(bytes);

        if(drawMode == VertexFormat.DrawMode.QUADS) {
//            int[] quadIndices = new int[] {0, 1, 3, 1, 2, 3};
            int[] quadIndices = new int[count * 6];

            for(int i=0;i<count;i++) {
                quadIndices[(i * 6)] = i * 4;
                quadIndices[(i * 6) + 1] = (i * 4) + 1;
                quadIndices[(i * 6) + 2] = (i * 4) + 3;
                quadIndices[(i * 6) + 3] = (i * 4) + 1;
                quadIndices[(i * 6) + 4] = (i * 4) + 2;
                quadIndices[(i * 6) + 5] = (i * 4) + 3;
            }

            WgpuNative.setIndexBuffer(quadIndices);
            WgpuNative.drawIndexed(count + (count / 2));
        } else if(drawMode == VertexFormat.DrawMode.TRIANGLES) {
            WgpuNative.draw(vertexCount);
        }

    }

}
