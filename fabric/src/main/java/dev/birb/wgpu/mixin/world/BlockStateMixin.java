package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.RustBlockStateAccessor;
import net.minecraft.block.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;

@Mixin(BlockState.class)
public class BlockStateMixin implements RustBlockStateAccessor {

    //This is just to make each BlockState instance have its own unique ID during testing if wgpu-mc isn't active and setting it for it
    @Unique
    private int rustBlockStateIndex = (int) (System.nanoTime() & 0xffffff);

    @Override
    public int wgpu_mc$getRustBlockStateIndex() {
        return this.rustBlockStateIndex;
    }

    @Override
    public void wgpu_mc$setRustBlockStateIndex(int l) {
        this.rustBlockStateIndex = l;
    }

}
