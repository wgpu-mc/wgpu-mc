package dev.birb.wgpu.mixin.world;

import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.registry.Registry;
import net.minecraft.world.HeightLimitView;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.UpgradeData;
import net.minecraft.world.gen.chunk.BlendingData;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Chunk.class)
public class ChunkMixin {

    @Shadow @Final protected ChunkSection[] sectionArray;

    @Inject(method = "<init>", at = @At("RETURN"))
    private void init(ChunkPos pos, UpgradeData upgradeData, HeightLimitView heightLimitView, Registry biome, long inhabitedTime, ChunkSection[] sectionArrayInitializer, BlendingData blendingData, CallbackInfo ci) {
        assert this.sectionArray.length == 24;

//        this.sectionArray[0].getBlockStateContainer().data;
    }

}
