package dev.birb.wgpu;

import dev.birb.wgpu.render.electrum.ElectrumRenderer;
import dev.birb.wgpu.rust.WgpuNative;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.renderer.v1.RendererAccess;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;

import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.text.LiteralText;
import org.lwjgl.glfw.GLFW;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class WgpuMcMod implements ClientModInitializer {

	public static Logger LOGGER = LoggerFactory.getLogger("wgpu_mc");

	public static ElectrumRenderer ELECTRUM;

	@Override
	public void onInitializeClient() {
		LOGGER.info("[Electrum] Registering Electrum renderer!");

		ElectrumRenderer electrumRenderer = new ElectrumRenderer();
		RendererAccess.INSTANCE.registerRenderer(electrumRenderer);

		KeyBinding binding = KeyBindingHelper.registerKeyBinding(
			new KeyBinding("", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_M, "")
		);

		ClientTickEvents.END_CLIENT_TICK.register(client -> {
			while (binding.wasPressed()) {
				PlayerEntity player = client.player;

				int light = player.world.getLightingProvider().getLight(player.getBlockPos(), 0);
				player.sendMessage(new LiteralText("Light at " + player.getBlockPos() + ": " + light), false);
			}
		});
	}
}
