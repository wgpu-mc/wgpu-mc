package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.RustPalette;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.nbt.NbtCompound;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.network.packet.s2c.play.ChunkData;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.registry.Registry;
import net.minecraft.world.HeightLimitView;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.EmptyChunk;
import net.minecraft.world.chunk.UpgradeData;
import net.minecraft.world.chunk.WorldChunk;
import net.minecraft.world.gen.chunk.BlendingData;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.function.Consumer;

@Mixin(WorldChunk.class)
public class WorldChunkMixin {

    @Inject(method = "loadFromPacket", at = @At("RETURN"))
    public void loadFromPacket(PacketByteBuf buf, NbtCompound nbt, Consumer<ChunkData.BlockEntityVisitor> consumer, CallbackInfo ci) {
        long[] palettePointers = new long[24];
        long[] storagePointers = new long[24];

        Chunk chunk = (Chunk) (Object) this;

        assert chunk.getSectionArray().length == 24;

        for(int i=0;i<24;i++) {
            RustPalette<?> rustPalette = (RustPalette<?>) chunk.getSection(i).getBlockStateContainer().data.palette;
            PaletteStorage paletteStorage = chunk.getSection(i).getBlockStateContainer().data.storage;

            palettePointers[i] = rustPalette.getRustPointer();

            if(paletteStorage instanceof PackedIntegerArray storage) {
                long rustPaletteStorage = WgpuNative.createPaletteStorage(storage.getData(), storage.elementsPerLong, storage.getElementBits(), storage.maxValue, storage.indexScale, storage.indexOffset, storage.indexShift, storage.getSize());
                storagePointers[i] = rustPaletteStorage;

                RustPalette.cleaner.register(storage, () -> WgpuNative.destroyPaletteStorage(rustPaletteStorage));
            }
        }

        WgpuNative.createChunk(0, 0, palettePointers, storagePointers);
    }

}
