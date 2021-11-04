package dev.birb.wgpu.mixin.disablers;

import dev.birb.wgpu.rust.Wgpu;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(Registry.class)
public class RegistryMixin {

    private static boolean INITIALIZED = false;

    @Inject(method = "register(Lnet/minecraft/util/registry/Registry;Lnet/minecraft/util/Identifier;Ljava/lang/Object;)Ljava/lang/Object;", at = @At("HEAD"), cancellable = true)
    private static void registryHook(Registry registry, Identifier id, Object entry, CallbackInfoReturnable<Object> cir) {
        if(!INITIALIZED) {
            System.load("/Users/birb/wgpu-mc/target/debug/libwgpu_mc_jni.dylib");
            Wgpu.initialize("Minecraft");
            INITIALIZED = true;
        }

        if(registry == Registry.BLOCK) {
//            Wgpu.registerEntry(0, id.toString());
        } else if(registry == Registry.ITEM) {
//            Wgpu.registerEntry(1, id.toString());
        }
    }

}
