package dev.birb.wgpu.mixin.disablers;

import org.lwjgl.system.Library;
import org.lwjgl.system.SharedLibrary;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(Library.class)
public class LibraryMixin {

//    @Inject(method = "loadNative(Ljava/lang/Class;Ljava/lang/String;ZZ)Lorg/lwjgl/system/SharedLibrary;", at = @At("HEAD"), cancellable = true)
//    private static void cancelLoadLibrary(Class<?> context, String name, boolean bundledWithLWJGL, boolean printError, CallbackInfoReturnable<SharedLibrary> cir) {
//        cir.cancel();
//    }

}
