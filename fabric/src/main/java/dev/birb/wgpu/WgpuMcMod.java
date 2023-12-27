package dev.birb.wgpu;


import dev.birb.wgpu.render.electrum.ElectrumRenderer;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.FabricKeyBinding;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.renderer.v1.RendererAccess;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.text.Text;
import net.minecraft.world.LightType;
import org.lwjgl.glfw.GLFW;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class WgpuMcMod implements ClientModInitializer {

	public static Logger LOGGER = LoggerFactory.getLogger("electrum");

	public static boolean ENTITIES_UPLOADED = false;
	public static boolean MAY_INJECT_PART_IDS = false;

	public static long TIME_SPENT_ENTITIES = 0;
	public static long ENTRIES = 0;

	@Override
	public void onInitializeClient() {
		LOGGER.info("Registering FRAPI renderer");

		ElectrumRenderer electrumRenderer = new ElectrumRenderer();
		RendererAccess.INSTANCE.registerRenderer(electrumRenderer);

		KeyBinding keyBinding = KeyBindingHelper.registerKeyBinding(new KeyBinding(
				"key.examplemod.spook", // The translation key of the keybinding's name
				InputUtil.Type.KEYSYM, // The type of the keybinding, KEYSYM for keyboard, MOUSE for mouse.
				GLFW.GLFW_KEY_M, // The keycode of the key
				"category.examplemod.test" // The translation key of the keybinding's category.
		));

		ClientTickEvents.END_CLIENT_TICK.register(client -> {
			while (keyBinding.wasPressed()) {
				int blockLightlevel = client.world.getLightLevel(LightType.BLOCK, client.player.getBlockPos());
				int skyLightlevel = client.world.getLightLevel(LightType.SKY, client.player.getBlockPos());
				client.player.sendMessage(Text.literal( skyLightlevel+ " " + blockLightlevel), false);
			}
		});
	}
}
