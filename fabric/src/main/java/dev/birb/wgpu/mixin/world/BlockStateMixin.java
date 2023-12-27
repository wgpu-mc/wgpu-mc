package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.RustBlockStateAccessor;
import net.minecraft.block.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;

@Mixin(BlockState.class)
public class BlockStateMixin implements RustBlockStateAccessor {

    private int rustBlockStateIndex = 0;

    @Override
    public int wgpu_mc$getRustBlockStateIndex() {
        return this.rustBlockStateIndex;
    }

    @Override
    public void wgpu_mc$setRustBlockStateIndex(int l) {
        this.rustBlockStateIndex = l;
    }

}
