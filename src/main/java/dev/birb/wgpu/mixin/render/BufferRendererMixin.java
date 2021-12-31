package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.Shader;
import net.minecraft.client.render.VertexFormat;
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
            WgpuNative.wmUsePipeline(0);
        } else if(vertexFormat == VertexFormats.POSITION_TEXTURE) {
            WgpuNative.wmUsePipeline(1);
        }

        int k;
        int j;
        Object indexBuffer;
        RenderSystem.assertOnRenderThread();
        buffer.clear();
        if (count <= 0) {
            return;
        }
        int i = count * vertexFormat.getVertexSize();
        BufferRenderer.bind(vertexFormat);
        buffer.position(0);
        buffer.limit(i);
        GlStateManager._glBufferData(34962, buffer, 35048);
        if (textured) {
            indexBuffer = RenderSystem.getSequentialBuffer(drawMode, vertexCount);
            j = ((RenderSystem.IndexBuffer)indexBuffer).getId();
            if (j != currentElementBuffer) {
                GlStateManager._glBindBuffer(34963, j);
                currentElementBuffer = j;
            }
            k = ((RenderSystem.IndexBuffer)indexBuffer).getElementFormat().count;
        } else {
            int indexBuffer2 = vertexFormat.getElementBuffer();
            if (indexBuffer2 != currentElementBuffer) {
                GlStateManager._glBindBuffer(34963, indexBuffer2);
                currentElementBuffer = indexBuffer2;
            }
            buffer.position(i);
            buffer.limit(i + vertexCount * elementFormat.size);
            GlStateManager._glBufferData(34963, buffer, 35048);
            k = elementFormat.count;
        }

//        Matrix4f projMat = RenderSystem.getProjectionMatrix();
        Matrix4f projMat = Matrix4f.projectionMatrix(1280.0f, 720.0f, 0.0f, 10000.0f);
        FloatBuffer projMatBuffer = FloatBuffer.allocate(16);
        projMat.readColumnMajor(projMatBuffer);
        WgpuNative.bindMatrix4f(0, projMatBuffer.array());

        GlStateManager._drawElements(drawMode.mode, vertexCount, k, 0L);
        buffer.position(0);
    }

}
