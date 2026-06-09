package dev.birb.wgpu.mixin.core;

import net.minecraft.core.Registry;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(Registry.class)
public interface RegistryMixin {
//    @Inject(method = "Lnet/minecraft/core/Registry;register(Lnet/minecraft/core/Registry;Lnet/minecraft/resources/ResourceKey;Ljava/lang/Object;)Ljava/lang/Object;", at = @At("RETURN"))
//    static void registryHook(Registry<?> registry, Identifier id, Object entry, CallbackInfoReturnable<Object> cir) {
//        if (entry instanceof Block block) {
//            String blockId = Registries.BLOCK.getId(block).toString();
//
//            WgpuNative.registerBlock(blockId);
//
//            for(BlockState state : block.getStateManager().getStates()) {
//                String stateKey = state.getEntries().entrySet().stream().map(net.minecraft.state.State.PROPERTY_MAP_PRINTER).collect(Collectors.joining(","));
//                WgpuNative.registerBlockState(state, blockId, stateKey);
//            }
//        }
//    }
}
