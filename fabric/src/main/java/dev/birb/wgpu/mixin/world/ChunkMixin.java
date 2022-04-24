package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.RustPalette;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
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

        long[] palettePointers = new long[24];
        long[] storagePointers = new long[24];

        for(int i=0;i<24;i++) {
            RustPalette<?> rustPalette = (RustPalette<?>) this.sectionArray[i].getBlockStateContainer().data.palette;
            PaletteStorage paletteStorage = this.sectionArray[i].getBlockStateContainer().data.storage;

            palettePointers[i] = rustPalette.getRustPointer();

            if(paletteStorage instanceof PackedIntegerArray storage) {
                long rustPaletteStorage = WgpuNative.createPaletteStorage(storage.getData(), storage.elementsPerLong, storage.getElementBits(), storage.maxValue, storage.indexScale, storage.indexOffset, storage.indexShift);
                storagePointers[i] = rustPaletteStorage;

                RustPalette.cleaner.register(storage, () -> WgpuNative.destroyPaletteStorage(rustPaletteStorage));
            }
        }

//        WgpuNative.setChunkOrigin(this);
        WgpuNative.createChunk(pos.x, pos.z, palettePointers, storagePointers);
    }

}
