package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.RustBlockStateAccessor;
import net.minecraft.block.BlockState;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(BlockState.class)
public class BlockStateMixin implements RustBlockStateAccessor {

    private int rustBlockStateIndex = (int) (System.nanoTime() & 0xffffff);

    @Override
    public int getRustBlockStateIndex() {
        return this.rustBlockStateIndex;
    }

    @Override
    public void setRustBlockStateIndex(int l) {
        this.rustBlockStateIndex = l;
    }

}
