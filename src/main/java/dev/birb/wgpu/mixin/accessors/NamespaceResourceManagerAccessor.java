package dev.birb.wgpu.mixin.accessors;

import net.minecraft.resource.NamespaceResourceManager;
import net.minecraft.resource.ResourcePack;
import net.minecraft.resource.ResourceType;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import java.util.List;

@Mixin(NamespaceResourceManager.class)
public interface NamespaceResourceManagerAccessor {

    @Accessor("packList")
    List<ResourcePack> getPackList();

    @Accessor("type")
    ResourceType getType();

}
