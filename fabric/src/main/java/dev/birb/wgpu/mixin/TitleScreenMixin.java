package dev.birb.wgpu.mixin;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.TitleScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TitleScreen.class)
public class TitleScreenMixin {

    @Unique
    private boolean updatedTitle = false;

    @Inject(method = "render", at = @At("HEAD"))
    private void render(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        if (!updatedTitle && Wgpu.isInitialized()) {
            WgpuNative.cacheBlockStates();
            MinecraftClient.getInstance().updateWindowTitle();
            updatedTitle = true;
        }
    }

}
