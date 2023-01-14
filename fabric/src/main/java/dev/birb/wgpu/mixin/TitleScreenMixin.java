package dev.birb.wgpu.mixin;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.util.math.MatrixStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import static dev.birb.wgpu.render.Wgpu.wmIdentity;
import static net.minecraft.client.gui.DrawableHelper.drawStringWithShadow;

@Mixin(TitleScreen.class)
public class TitleScreenMixin {

    private boolean updatedTitle = false;

    @Inject(method = "render", at = @At("HEAD"))
    private void render(MatrixStack matrices, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        if(!updatedTitle && Wgpu.INITIALIZED) {

            WgpuNative.cacheBlockStates();
            Wgpu.uploadBlockLayers();

            MinecraftClient.getInstance().updateWindowTitle();
            updatedTitle = true;
        }
    }

}
