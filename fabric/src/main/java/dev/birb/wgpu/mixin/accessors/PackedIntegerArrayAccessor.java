package dev.birb.wgpu.mixin.accessors;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.util.collection.PackedIntegerArray;

@Mixin(PackedIntegerArray.class)
public interface PackedIntegerArrayAccessor {
    
    @Accessor
    long getMaxValue();

    @Accessor
    int getElementsPerLong();

    @Accessor
    int getIndexScale();

    @Accessor
    int getIndexOffset();

    @Accessor
    int getIndexShift();

}
