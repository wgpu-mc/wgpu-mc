package dev.birb.wgpu.mixin.render;

import net.minecraft.client.texture.SpriteAtlasTexture;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(SpriteAtlasTexture.class)
public class SpriteAtlasTextureMixin {

    // This prevents animated sprites from ticking, which uploads lots of unnecessary data to the GPu
    @Overwrite
    public void tick() {

    }
}
