package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.game.MainGameThread;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.RunArgs;
import net.minecraft.client.gui.screen.Screen;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(MinecraftClient.class)
public abstract class MinecraftClientCoreMixin {

    @Shadow
    protected abstract String getWindowTitle();

    @Shadow @Nullable
    public Screen currentScreen;

//    @Inject(method = "openScreen", at = @At("HEAD"), cancellable = true)
//    public void openScreen(Screen screen, CallbackInfo ci) {
//        screen.init((MinecraftClient) (Object) this, 1280, 720);
//        this.currentScreen = screen;
//        ci.cancel();
//    }

    //
    @Inject(method = "updateWindowTitle", at = @At("HEAD"), cancellable = true)
    public void modifyUpdateWindowTitle(CallbackInfo ci) {
        WgpuNative.updateWindowTitle(this.getWindowTitle());
        ci.cancel();
    }

    @Inject(method = "getWindowTitle", at = @At(value = "RETURN"), cancellable = true)
    public void getWindowTitleAddWgpu(CallbackInfoReturnable<String> cir) {
        String title = cir.getReturnValue();
        if(!Wgpu.INITIALIZED) {
            title += " + Wgpu";
        } else {
            title += " + " + WgpuNative.getBackend();
        }
        cir.setReturnValue(title);
    }

    @Inject(method = "run", at = @At("HEAD"))
    public void injectRun(CallbackInfo ci) {
        Wgpu.initRenderer((MinecraftClient) (Object) this);
    }

    @Inject(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/thread/ReentrantThreadExecutor;<init>(Ljava/lang/String;)V", shift = At.Shift.AFTER))
    private void injectPreInit(RunArgs args, CallbackInfo ci) {
        System.out.println("Initializing wgpu-mc");
        Wgpu.preInit("Minecraft");
    }

//    @Inject(method = "<init>", at = @At("TAIL"))
//    public void injectWindowHook(RunArgs args, CallbackInfo ci) {
//        Wgpu.initRenderer((MinecraftClient) (Object) this);
//    }

}
