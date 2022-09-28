package dev.birb.wgpu.rust;

import dev.birb.wgpu.mixin.accessors.PackedIntegerArrayAccessor;
import dev.birb.wgpu.palette.RustPalette;
import net.minecraft.client.MinecraftClient;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.collection.PaletteStorage;
import net.minecraft.world.chunk.WorldChunk;

public class WmChunk {
    public WorldChunk worldChunk;
    public int x;
    public int z;

    public WmChunk(WorldChunk worldChunk) {
        MinecraftClient client = MinecraftClient.getInstance();

        this.x = worldChunk.getPos().x - client.player.getChunkPos().x;
        this.z = worldChunk.getPos().z - client.player.getChunkPos().z;

        this.worldChunk = worldChunk;
    }

    public void upload() throws ClassCastException {
        long[] palettePointers = new long[24];
        long[] storagePointers = new long[24];

        assert this.worldChunk.getSectionArray().length == 24;

        for(int i=0;i<24;i++) {
            RustPalette<?> rustPalette = (RustPalette<?>) this.worldChunk.getSection(i).getBlockStateContainer().data.palette;
            PaletteStorage paletteStorage = this.worldChunk.getSection(i).getBlockStateContainer().data.storage;

            palettePointers[i] = rustPalette.getRustPointer();

            if(paletteStorage instanceof PackedIntegerArrayAccessor accessor) {
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

                storagePointers[i] = pointer;
            }
        }

        WgpuNative.createChunk(this.x, this.z, palettePointers, storagePointers);
    }

    public void bake() {
        //Non-blocking
        WgpuNative.bakeChunk(this.x, this.z);
    }

}
