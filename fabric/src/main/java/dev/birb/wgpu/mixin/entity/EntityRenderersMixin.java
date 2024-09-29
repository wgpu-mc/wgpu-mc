package dev.birb.wgpu.mixin.entity;

import com.google.common.collect.ImmutableMap;
import dev.birb.wgpu.entity.EntityState;
import net.minecraft.client.render.entity.EntityRendererFactory;
import net.minecraft.client.render.entity.EntityRenderers;
import net.minecraft.entity.EntityType;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(EntityRenderers.class)
public class EntityRenderersMixin {

    @Inject(method = "method_32174", at = @At(value = "INVOKE", target = "Lcom/google/common/collect/ImmutableMap$Builder;put(Ljava/lang/Object;Ljava/lang/Object;)Lcom/google/common/collect/ImmutableMap$Builder;", shift = At.Shift.BEFORE))
    private static void reloadEntityRenderers(ImmutableMap.Builder builder, EntityRendererFactory.Context context, EntityType entityType, EntityRendererFactory factory, CallbackInfo ci) {
        EntityState.builderType = entityType;
        EntityState.registeringRoot = true;
    }

}
