package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.gui.OptionPageScreen;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import org.spongepowered.asm.mixin.Dynamic;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(OptionsScreen.class)
public class OptionsScreenMixin {
    @Dynamic
    @Inject(method = "method_19828", at = @At("HEAD"), cancellable = true)
    private void onOpenVideoOptionsScreen(CallbackInfoReturnable<Screen> cir) {
        cir.setReturnValue(new OptionPageScreen((Screen) (Object) this));
    }
}
