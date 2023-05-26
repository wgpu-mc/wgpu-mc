package dev.birb.wgpu.mixin;

import com.google.gson.Gson;
import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.model.TexturedModelData;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.render.entity.model.EntityModelLoader;
import net.minecraft.client.render.entity.model.EntityModels;
import net.minecraft.client.texture.TextureManager;
import net.minecraft.client.util.math.MatrixStack;
import org.apache.logging.log4j.Logger;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

import static dev.birb.wgpu.render.Wgpu.wmIdentity;
import static net.minecraft.client.gui.DrawableHelper.drawStringWithShadow;
import static net.minecraft.screen.PlayerScreenHandler.BLOCK_ATLAS_TEXTURE;

@Mixin(TitleScreen.class)
public class TitleScreenMixin {

    @Shadow @Final private static Logger LOGGER;
    private boolean updatedTitle = false;

    @Inject(method = "render", at = @At("HEAD"))
    private void render(MatrixStack matrices, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        if(!updatedTitle && Wgpu.INITIALIZED) {
            WgpuNative.cacheBlockStates();
            MinecraftClient.getInstance().updateWindowTitle();
            updatedTitle = true;

            long millis = System.currentTimeMillis();

            StringBuilder builder = new StringBuilder();

            builder.append("{");
            boolean first = true;

            Map<EntityModelLayer, TexturedModelData> models = EntityModels.getModels();

            for(Map.Entry<EntityModelLayer, TexturedModelData> entry : models.entrySet()) {
                if(!first) {
                    builder.append(",");
                }
                first = false;
                String dumped = (new Gson()).toJson(entry.getValue());
                builder.append("\"").append(entry.getKey()).append("\":");
                builder.append(dumped);
            }
            builder.append("}");

            WgpuNative.registerEntities(builder.toString());

            WgpuMcMod.MAY_INJECT_PART_IDS = true;

            WgpuMcMod.LOGGER.info("Uploaded " + models.size() + " TMDs to wgpu-mc and processed them in " + (System.currentTimeMillis() - millis) + "ms");

            TextureManager textureManager = MinecraftClient.getInstance().getTextureManager();
            int blockTexAtlasId = textureManager.getTexture(BLOCK_ATLAS_TEXTURE).getGlId();

            WgpuNative.identifyGlTexture(0, blockTexAtlasId);

            WgpuMcMod.ENTITIES_UPLOADED = true;
        }
    }

}
