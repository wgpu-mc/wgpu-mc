package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.texture.Sprite;
import net.minecraft.client.texture.SpriteAtlasTexture;
import net.minecraft.client.texture.TextureStitcher;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Collection;
import java.util.List;
import java.util.Set;

@Mixin(SpriteAtlasTexture.class)
public class SpriteAtlasTextureMixin {

    @Inject(method = "loadSprites(Lnet/minecraft/resource/ResourceManager;Lnet/minecraft/client/texture/TextureStitcher;I)Ljava/util/List;", at = @At("HEAD"), cancellable = true)
    private void loadSprites0(ResourceManager resourceManager, TextureStitcher textureStitcher, int maxLevel, CallbackInfoReturnable<List<Sprite>> cir) {
        cir.cancel();
    }

    @Inject(method = "loadSprites(Lnet/minecraft/resource/ResourceManager;Ljava/util/Set;)Ljava/util/Collection;", at = @At("HEAD"), cancellable = true)
    private void loadSprites1(ResourceManager resourceManager, Set<Identifier> ids, CallbackInfoReturnable<Collection<Sprite.Info>> cir) {
        cir.cancel();
    }

    @Inject(method = "loadSprites(Lnet/minecraft/resource/ResourceManager;Lnet/minecraft/client/texture/TextureStitcher;I)Ljava/util/List;", at = @At("HEAD"), cancellable = true)
    private void loadSprites2(ResourceManager resourceManager, TextureStitcher textureStitcher, int maxLevel, CallbackInfoReturnable<List<Sprite>> cir) {
        cir.cancel();
    }

}
