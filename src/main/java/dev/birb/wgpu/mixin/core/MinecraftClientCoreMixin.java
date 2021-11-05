package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.game.MainGameThread;
import dev.birb.wgpu.rust.Wgpu;
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

    @Inject(method = "openScreen", at = @At("HEAD"), cancellable = true)
    public void openScreen(Screen screen, CallbackInfo ci) { //TODO: Temporary!
        this.currentScreen = screen;
        ci.cancel();
    }

    //
    @Inject(method = "updateWindowTitle", at = @At("HEAD"), cancellable = true)
    public void modifyUpdateWindowTitle(CallbackInfo ci) {
        Wgpu.updateWindowTitle(this.getWindowTitle());
        ci.cancel();
    }

    @Inject(method = "getWindowTitle", at = @At(value = "RETURN"), cancellable = true)
    public void getWindowTitleAddWgpu(CallbackInfoReturnable<String> cir) {
        cir.setReturnValue(cir.getReturnValue() + " + Wgpu");
    }

    @Inject(method = "run", at = @At("HEAD"))
    public void injectRun(CallbackInfo ci) {
    }

    @Inject(method = "<init>", at = @At("TAIL"))
    public void injectWindowHook(RunArgs args, CallbackInfo ci) {
        MainGameThread.createNewThread((MinecraftClient) (Object) this);

        //Initializing the window hijacks this thread to run the event loop. All communication is now done through channels.
        Wgpu.doEventLoop();
    }

}
