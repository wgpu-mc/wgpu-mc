package dev.birb.wgpu.backend;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.vertex.VertexFormat;
import dev.birb.wgpu.rust.WgpuNative;
import lombok.Getter;
import net.minecraft.client.gl.RenderPipelines;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.system.libc.LibCString;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.Collection;
import java.util.function.Supplier;

public class WgpuRenderPass implements RenderPass {
    
    @Getter
    private final ByteBuffer commands = MemoryUtil.memCalloc(16000);
    @Getter
    private int commandCount = 0;
    
    @Getter
    private WgpuTexture target;
    @Getter
    private WgpuTexture depth;

    private static final int COMMAND_SIZE = (int) WgpuNative.getRenderPassCommandSize();
    
    public WgpuRenderPass(WgpuTexture target, WgpuTexture depth) {
        this.target = target;
        this.depth = depth;
    }
    
    @Override
    public void pushDebugGroup(Supplier<String> supplier) {

    }

    @Override
    public void popDebugGroup() {

    }

    @Override
    public void setPipeline(RenderPipeline pipeline) {
        int pipelineId = 0;

        int p = commands.position();

        commands.putLong(4);

        commands.position(p + COMMAND_SIZE);

        if(pipeline == RenderPipelines.GUI_TEXTURED) pipelineId = 1;

        commands.putInt(pipelineId);

        return;
    }

    @Override
    public void bindSampler(String name, @Nullable GpuTexture texture) {
        byte[] nameBytes = name.getBytes(StandardCharsets.UTF_8);
        ByteBuffer nameStr = MemoryUtil.memCalloc(nameBytes.length);
        nameStr.put(nameBytes);

        int p = commands.position();

        commands.putLong(5);
        commands.putLong(((WgpuTexture) texture).getTexture());
        commands.putLong(MemoryUtil.memAddress0(nameStr));
        commands.putInt(nameBytes.length);

        commands.position(p + COMMAND_SIZE);
    }

    @Override
    public void setUniform(String name, GpuBuffer buffer) {
        setUniform(name, buffer.slice());
    }

    @Override
    public void setUniform(String name, GpuBufferSlice slice) {
        byte[] nameBytes = name.getBytes(StandardCharsets.UTF_8);
        ByteBuffer nameStr = MemoryUtil.memCalloc(nameBytes.length);
        nameStr.put(nameBytes);

        int p = commands.position();

        commands.putLong(6);
        commands.putLong(((WgpuBuffer) slice.buffer()).getWgpuBuffer());
        commands.putLong(MemoryUtil.memAddress0(nameStr));
        commands.putInt(nameBytes.length);
        commands.putInt(slice.offset());
        commands.putInt(slice.length() + slice.offset());

        commands.position(p + COMMAND_SIZE);
    }

    @Override
    public void enableScissor(int x, int y, int width, int height) {

    }

    @Override
    public void disableScissor() {

    }

    @Override
    public void setVertexBuffer(int index, GpuBuffer buffer) {
        int p = commands.position();

        commands.putLong(3);
        commands.putLong(((WgpuBuffer) buffer).getWgpuBuffer());
        commands.putInt(index);

        commands.position(p + COMMAND_SIZE);

        commandCount++;
    }

    @Override
    public void setIndexBuffer(GpuBuffer indexBuffer, VertexFormat.IndexType indexType) {
        int i = switch(indexType) {
            case VertexFormat.IndexType.SHORT -> 0;
            case VertexFormat.IndexType.INT -> 1;
        };

        int p = commands.position();
        
        commands.putLong(2);
        commands.putLong(((WgpuBuffer) indexBuffer).getWgpuBuffer());
        commands.putInt(i);

        commands.position(p + COMMAND_SIZE);
        commandCount++;
    }

    @Override
    public void drawIndexed(int offset, int count, int primcount, int i) {
        int p = commands.position();

        commands.putLong(1);
        commands.putInt(offset);
        commands.putInt(count);
        commands.putInt(primcount);
        commands.putInt(i);

        commands.position(p + COMMAND_SIZE);
        commandCount++;
    }

    @Override
    public void drawMultipleIndexed(Collection<RenderObject> objects, @Nullable GpuBuffer buffer, @Nullable VertexFormat.IndexType indexType, Collection<String> validationSkippedUniforms) {

    }

    @Override
    public void draw(int offset, int count) {
        int p = commands.position();

        commands.putLong(0);
        commands.putInt(offset);
        commands.putInt(count);

        commands.position(p + COMMAND_SIZE);

        commandCount++;
    }

    @Override
    public void close() {

    }
}
