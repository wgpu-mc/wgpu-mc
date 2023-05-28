package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.network.packet.s2c.play.LightData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Arrays;
import java.util.BitSet;

@Mixin(LightData.class)
public abstract class LightDataMixin {

    @Shadow public abstract BitSet getInitedSky();

    @Shadow public abstract boolean isNonEdge();

    private int readerIndex;

    @Inject(method = "<init>(Lnet/minecraft/network/PacketByteBuf;II)V", at = @At(value = "INVOKE", target = "Lnet/minecraft/network/PacketByteBuf;readBoolean()Z", shift = At.Shift.BEFORE))
    private void bindLight(PacketByteBuf buf, int x, int z, CallbackInfo ci) {
        long lightData = WgpuNative.createAndDeserializeLightData(buf.array(), buf.arrayOffset() + buf.readerIndex());

        WgpuNative.bindLightData(lightData, x, z);
    }

}
