package dev.birb.wgpu.mixin.accessors;

import net.minecraft.resource.NamespaceResourceManager;
import net.minecraft.resource.ReloadableResourceManagerImpl;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import java.util.Map;

@Mixin(ReloadableResourceManagerImpl.class)
public interface ReloadableResourceManagerImplAccessor {

    @Accessor("namespaceManagers")
    Map<String, NamespaceResourceManager> getNamespaceManagers();

}
