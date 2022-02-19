package dev.birb.wgpu.mixin.core;

import net.minecraft.util.registry.Registry;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(Registry.class)
public class RegistryMixin {

//    private static boolean INITIALIZED = false;
//
//    @Inject(method = "register(Lnet/minecraft/util/registry/Registry;Lnet/minecraft/util/Identifier;Ljava/lang/Object;)Ljava/lang/Object;", at = @At("HEAD"), cancellable = true)
//    private static void registryHook(Registry registry, Identifier id, Object entry, CallbackInfoReturnable<Object> cir) {
//        if(!INITIALIZED) {
//            System.load("/Users/birb/wgpu-mc/target/debug/libwgpu_mc_jni.dylib");
//            Wgpu.initialize("Minecraft");
//            INITIALIZED = true;
//        }
//    }

}
