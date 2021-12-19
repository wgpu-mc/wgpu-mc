package dev.birb.wgpu.mixin.render;

import com.mojang.datafixers.util.Pair;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.util.math.Matrix4f;
import org.lwjgl.opengl.GL20;
import org.lwjgl.opengl.GL44;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;
import java.nio.FloatBuffer;

@Mixin(VertexBuffer.class)
public class VertexBufferMixin {

    @Shadow private int id;

    @Shadow private int vertexCount;

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void draw(Matrix4f matrix, int mode) {
//        FloatBuffer buffer = FloatBuffer.allocate(4 * 4);
//        matrix.writeToBuffer(buffer);
//        int matrix_id = WgpuNative.uploadBuffer(MemoryUtil.memAddress(buffer), buffer.capacity() * 4L, 0);
//        WgpuNative.bindBuffer(GL44.GL_ARRAY_BUFFER, this.id);
//        WgpuNative.bindBuffer(GL44.GL_UNIFORM_BUFFER, matrix_id);
//        WgpuNative.drawArray(mode, 0, this.vertexCount);
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void uploadInternal(BufferBuilder buffer) {
//        Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> pair = buffer.popData();
//        //TODO: impl usage
//        this.id = WgpuNative.uploadBuffer(
//                MemoryUtil.memAddress(pair.getSecond()),
//                0,
//                0);
    }

}
