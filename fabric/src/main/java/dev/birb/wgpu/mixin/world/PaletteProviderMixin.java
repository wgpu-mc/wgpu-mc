package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.SubvertedPaletteProvider;
import net.minecraft.world.chunk.PalettedContainer;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(PalettedContainer.PaletteProvider.class)
public class PaletteProviderMixin {

    @Mutable
    @Shadow @Final public static PalettedContainer.PaletteProvider BLOCK_STATE;

    @Inject(method = "<clinit>", at = @At("RETURN"))
    private static void clinit(CallbackInfo info) {
        BLOCK_STATE = SubvertedPaletteProvider.PROVIDER;
    }

}
