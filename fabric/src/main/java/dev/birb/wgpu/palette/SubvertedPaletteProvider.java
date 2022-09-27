package dev.birb.wgpu.palette;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.world.chunk.PalettedContainer;
import net.minecraft.world.chunk.SingularPalette;

public class SubvertedPaletteProvider {

    public static PalettedContainer.PaletteProvider PROVIDER = new PalettedContainer.PaletteProvider(4){

        @Override
        public <A> PalettedContainer.DataProvider<A> createDataProvider(IndexedIterable<A> idList, int bits) {
            // if(bits == 0) {
            //     return new PalettedContainer.DataProvider<>(SingularPalette::create, bits);
            // }
            return new PalettedContainer.DataProvider<>(RustPalette::create, bits);
        }

    };

}
