package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.rust.WmChunk;
import net.minecraft.nbt.NbtCompound;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.network.packet.s2c.play.ChunkData;
import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.function.Consumer;

@Mixin(WorldChunk.class)
public abstract class WorldChunkMixin {
    @Inject(method = "loadFromPacket", at = @At("RETURN"))
    public void loadFromPacket(PacketByteBuf buf, NbtCompound nbt, Consumer<ChunkData.BlockEntityVisitor> consumer, CallbackInfo ci) {
        WmChunk chunk = new WmChunk((WorldChunk) (Object) this);
        try {
            chunk.uploadAndBake();
        } catch(ClassCastException e) {
            WgpuMcMod.LOGGER.error("Could not upload and bake chunk", e);
        }
    }

}
