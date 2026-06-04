package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.helper.RustBlockStateAccessor;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;

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
