package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.RustBlockStateAccessor;
import net.minecraft.block.BlockState;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(BlockState.class)
public class BlockStateMixin implements RustBlockStateAccessor {

    private long rustBlockStateIndex;

    @Override
    public long getRustBlockStateIndex() {
        return this.rustBlockStateIndex;
    }

    @Override
    public void setRustBlockStateIndex(long l) {
        this.rustBlockStateIndex = l;
    }

}
