package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.mixin.accessors.PackedIntegerArrayAccessor;
import dev.birb.wgpu.palette.RustPalette;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.block.BlockState;
import net.minecraft.client.world.ClientWorld;
import net.minecraft.nbt.NbtCompound;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.network.packet.s2c.play.ChunkData;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.World;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.function.Consumer;

@Mixin(WorldChunk.class)
public abstract class WorldChunkMixin {

    @Shadow @Final private World world;

    @Shadow public abstract BlockState getBlockState(BlockPos pos);

    @Inject(method = "loadFromPacket", at = @At("RETURN"))
    public void loadFromPacket(PacketByteBuf buf, NbtCompound nbt, Consumer<ChunkData.BlockEntityVisitor> consumer, CallbackInfo ci) {
        long[] palettePointers = new long[24];
        long[] storagePointers = new long[24];

        Chunk chunk = (Chunk) (Object) this;

        ChunkPos pos = ((Chunk) (Object) this).getPos();

        assert chunk.getSectionArray().length == 24;

        for(int i=0;i<24;i++) {
            // RustPalette<?> rustPalette = (RustPalette<?>) chunk.getSection(i).getBlockStateContainer().data.palette;
            PaletteStorage paletteStorage = chunk.getSection(i).getBlockStateContainer().data.storage;

            // palettePointers[i] = rustPalette.getRustPointer();
            if(paletteStorage instanceof PackedIntegerArray packedIntegerArray) {
                PackedIntegerArrayAccessor accessor = (PackedIntegerArrayAccessor) (Object) paletteStorage;

                long pointer = WgpuNative.createPaletteStorage(
                    paletteStorage.getData(), 
                    accessor.getElementsPerLong(), 
                    paletteStorage.getElementBits(), 
                    accessor.getMaxValue(), 
                    accessor.getIndexScale(), 
                    accessor.getIndexOffset(), 
                    accessor.getIndexShift(), 
                    paletteStorage.getSize()
                );

                assert packedIntegerArray.get(0) == WgpuNative.piaGetByIndex(pointer, 0);
                // System.out.println(packedIntegerArray.get(1) + " / " + WgpuNative.piaGetByIndex(pointer, 1));
                for(int a=0;a<packedIntegerArray.getSize();a++) {
                    int wmVal = WgpuNative.piaGetByIndex(pointer, a);
                    assert packedIntegerArray.get(a) == wmVal;
                    if(pos.x % 20 == 0 && pos.z % 20 == 0) System.out.println(chunk.getSection(i).getBlockStateContainer().data.palette.get(wmVal));
                }
 
                storagePointers[i] = pointer;
            }
        }

        // chunk.getBlockState(new BlockPos(0, 0, 0));

        int originX = ((ClientWorld) this.world).getChunkManager().chunks.centerChunkX;
        int originZ = ((ClientWorld) this.world).getChunkManager().chunks.centerChunkX;

        // WgpuNative.createChunk(pos.x - originX, pos.z - originZ, palettePointers, storagePointers);
    }

}
