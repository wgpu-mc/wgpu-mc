package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.rust.Wgpu;
import net.minecraft.client.world.ClientChunkManager;
import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(targets = { "net.minecraft.client.world.ClientChunkManager$ClientChunkMap" })
public class ClientChunkMapMixin {

    @Inject(
            method = "set",
            at = @At("HEAD")
    )
    protected void set(int index, WorldChunk chunk, CallbackInfo ci) {
        long time = System.currentTimeMillis();
        Wgpu.uploadChunk(chunk);
        System.out.println(System.currentTimeMillis() - time);
    }

}
