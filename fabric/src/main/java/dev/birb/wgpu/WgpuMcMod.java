package dev.birb.wgpu;

import net.fabricmc.api.ClientModInitializer;
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
		System.loadLibrary("renderdoc");
//		ResourceManagerHelper.get(PackType.CLIENT_RESOURCES).registerReloadListener(new ShaderReloadListener());
	}
}
