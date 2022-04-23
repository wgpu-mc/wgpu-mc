package dev.birb.wgpu.palette;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.util.collection.IndexedIterable;
import net.minecraft.util.math.MathHelper;
import net.minecraft.world.chunk.IdListPalette;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PaletteResizeListener;
import net.minecraft.world.chunk.PalettedContainer;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;

import java.util.List;

public class SubvertedPaletteProvider {

    public static PalettedContainer.PaletteProvider PROVIDER = new PalettedContainer.PaletteProvider(4){

        @Override
        public <A> PalettedContainer.DataProvider<A> createDataProvider(IndexedIterable<A> idList, int bits) {
            return new PalettedContainer.DataProvider<>(new Palette.Factory() {
                @Override
                public <A> Palette<A> create(int bits, IndexedIterable<A> idList, PaletteResizeListener<A> listener, List<A> list) {
                    return new RustPalette<>(WgpuNative.uploadIdList(idList));
                }
            }, bits);
        }
    };

}
