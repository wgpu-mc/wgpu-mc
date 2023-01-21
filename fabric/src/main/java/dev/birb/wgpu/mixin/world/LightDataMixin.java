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

    @Redirect(method = "<init>(Lnet/minecraft/network/PacketByteBuf;II)V", at = @At(value = "INVOKE", target = "Lnet/minecraft/network/PacketByteBuf;readBoolean()Z"))
    public boolean setIndex(PacketByteBuf instance) {
        this.readerIndex = instance.readerIndex();
        return instance.readBoolean();
    }

    @Inject(method = "<init>(Lnet/minecraft/network/PacketByteBuf;II)V", at = @At("RETURN"))
    private void readPacket(PacketByteBuf buf, int x, int y, CallbackInfo ci) {
        int index = readerIndex;
        long lightData = WgpuNative.createAndDeserializeLightData(buf.array(), index);
        LightData ld = (LightData) (Object) this;

        System.out.println("non_edge: " + this.isNonEdge() + " inited_sky: " + Arrays.toString(this.getInitedSky().toLongArray()) + " sky_nibbles: " + ld.getSkyNibbles());
    }

}
