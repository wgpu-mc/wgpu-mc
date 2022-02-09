package dev.birb.wgpu.mixin.accessors;

import net.minecraft.client.resource.DefaultClientResourcePack;
import net.minecraft.client.resource.ResourceIndex;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(DefaultClientResourcePack.class)
public interface DefaultClientResourcePackAccessor {

    @Accessor("index")
    ResourceIndex getResourceIndex();

}
