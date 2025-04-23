package dev.birb.wgpu;


import dev.birb.wgpu.render.ShaderReloadListener;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.resource.ResourceManagerHelper;
import net.minecraft.resource.ResourceType;
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
		ResourceManagerHelper.get(ResourceType.CLIENT_RESOURCES).registerReloadListener(new ShaderReloadListener());
	}
}
