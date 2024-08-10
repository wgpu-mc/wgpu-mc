package dev.birb.wgpu.mixin.world;

import net.minecraft.network.packet.s2c.play.LightData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import java.util.Arrays;
import java.util.BitSet;

@Mixin(LightData.class)
public abstract class LightDataMixin {

    @Shadow public abstract BitSet getInitedSky();

    @Shadow public abstract boolean isNonEdge();

    private int readerIndex;


}
