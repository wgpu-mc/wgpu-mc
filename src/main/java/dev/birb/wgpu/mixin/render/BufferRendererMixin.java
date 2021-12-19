package dev.birb.wgpu.mixin.render;

import dev.birb.wgpu.mixin.accessors.VertexFormatElementAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {


    /**
     * @author wgpu-mc
     */
    @Overwrite
    public static void draw(ByteBuffer buffer, int mode, VertexFormat vertexFormat, int count) {
        WgpuNative.drawArray(mode, 0, count);
    }

    private static long packVertexCounts(VertexFormat format) {
        long out = 0;
        int loc = 0;
        for(VertexFormatElement element : format.getElements()) {
            int count = ((VertexFormatElementAccessor) element).getCount();
            out |= ((long) count & 0xff) << (loc * 8);
            loc++;
        }
        return out;
    }

    private static long packVertexFormat(VertexFormat format) {
        long out = 0;
        int loc = 0;
        for(VertexFormatElement element : format.getElements()) {
            if(element.getFormat() == VertexFormatElement.Format.BYTE) {
                out |= 0b1L << (loc * 8);
            } else if(element.getFormat() == VertexFormatElement.Format.FLOAT) {
                out |= 0b10L << (loc * 8);
            } else if(element.getFormat() == VertexFormatElement.Format.INT) {
                out |= 0b100L << (loc * 8);
            } else if(element.getFormat() == VertexFormatElement.Format.SHORT) {
                out |= 0b1000L << (loc * 8);
            } else if (element.getFormat() == VertexFormatElement.Format.UBYTE) {
                out |= 0b10000L << (loc * 8);
            } else if (element.getFormat() == VertexFormatElement.Format.UINT) {
                out |= 0b100000L << (loc * 8);
            } else if (element.getFormat() == VertexFormatElement.Format.USHORT) {
                out |= 0b1000000L << (loc * 8);
            }
            loc++;
        }
        return out;
    }

    private static long packVertexType(VertexFormat format) {
        long out = 0;
        int loc = 0;
        for(VertexFormatElement element : format.getElements()) {
            if(element.getType() == VertexFormatElement.Type.POSITION) {
                out |= 0b1L << (loc * 8);
            } else if(element.getType() == VertexFormatElement.Type.COLOR) {
                out |= 0b10L << (loc * 8);
            } else if(element.getType() == VertexFormatElement.Type.GENERIC) {
                out |= 0b100L << (loc * 8);
            } else if(element.getType() == VertexFormatElement.Type.NORMAL) {
                out |= 0b1000L << (loc * 8);
            } else if (element.getType() == VertexFormatElement.Type.UV) {
                out |= 0b10000L << (loc * 8);
            }
            loc++;
        }
        return out;
    }

}
