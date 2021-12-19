package dev.birb.wgpu.mixin.accessors;

import net.minecraft.client.render.VertexFormatElement;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(VertexFormatElement.class)
public interface VertexFormatElementAccessor {

    @Accessor("count")
    int getCount();

}
