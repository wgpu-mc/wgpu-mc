package dev.birb.wgpu.mixin.world;

import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.world.chunk.PalettedContainer;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;

public class SubvertedPaletteProvider {

    public static PalettedContainer.PaletteProvider PROVIDER = new PalettedContainer.PaletteProvider(4) {

        @Override
        public <A> PalettedContainer.DataProvider<A> createDataProvider(IndexedIterable<A> idList, int bits) {
            return new PalettedContainer.DataProvider(BI_MAP, bits);
        }

    };

}
