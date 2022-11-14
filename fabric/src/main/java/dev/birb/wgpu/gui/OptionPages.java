package dev.birb.wgpu.gui;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import dev.birb.wgpu.gui.options.BoolOption;
import dev.birb.wgpu.gui.options.EnumOption;
import dev.birb.wgpu.gui.options.IntOption;
import dev.birb.wgpu.gui.options.Option;
import dev.birb.wgpu.gui.options.RustOptionInfo;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.AoMode;
import net.minecraft.client.option.AttackIndicator;
import net.minecraft.client.option.CloudRenderMode;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.option.GraphicsMode;
import net.minecraft.client.option.ParticlesMode;
import net.minecraft.text.LiteralText;
import net.minecraft.text.Text;
import net.minecraft.text.TranslatableText;
import org.jetbrains.annotations.NotNull;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;

public class OptionPages implements Iterable<OptionPages.Page> {
	private static final TypeToken<Map<String, RustOptionInfo>> SETTINGS_STRUCTURE_TYPE_TOKEN = new TypeToken<>() {
	};
	private static final TypeToken<List<Option<?>>> SETTINGS_TYPE_TOKEN = new TypeToken<>() {
	};
	private static final Gson GSON = new GsonBuilder()
			.registerTypeAdapter(SETTINGS_TYPE_TOKEN.getType(), new Option.OptionDeserializer())
			.create();
	public static final Map<String, RustOptionInfo> SETTINGS_STRUCTURE = GSON.fromJson(
			WgpuNative.getSettingsStructure(),
			SETTINGS_STRUCTURE_TYPE_TOKEN.getType());
	private final List<Page> pages = new ArrayList<>();

	public OptionPages() {
		pages.add(createGeneral());
		pages.add(createElectrum());
		pages.add(createQuality());
	}

	public Page getDefault() {
		return pages.get(0);
	}

	public boolean isChanged() {
		for (Page page : pages) {
			if (page.isChanged()) return true;
		}

		return false;
	}

	public void apply() {
		for (Page page : pages) page.apply();
	}

	public void undo() {
		for (Page page : pages) page.undo();
	}

	@NotNull
	@Override
	public Iterator<Page> iterator() {
		return pages.iterator();
	}

	private Page createGeneral() {
		Page page = new Page(new LiteralText("General"));

		MinecraftClient mc = MinecraftClient.getInstance();
		GameOptions options = mc.options;

		// 1
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.renderDistance"))
				.setAccessors(() -> options.viewDistance, integer -> {
					options.viewDistance = integer;
					mc.worldRenderer.scheduleTerrainUpdate();
				})
				.setFormatter(integer -> new TranslatableText("options.chunks", integer))
				.setRange(2, 32)
				.build()
		);
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.simulationDistance"))
				.setAccessors(() -> options.simulationDistance, integer -> options.simulationDistance = integer)
				.setFormatter(integer -> new TranslatableText("options.chunks", integer))
				.setRange(5, 16)
				.build()
		);
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.gamma"))
				.setAccessors(() -> (int) (options.gamma * 100), integer -> options.gamma = integer / 100.0)
				.setFormatter(integer -> {
					if (integer == 0) return new TranslatableText("options.gamma.min");
					else if (integer == 50) return new TranslatableText("options.gamma.default");
					else if (integer == 100) return new TranslatableText("options.gamma.max");

					return new LiteralText(integer + "%");
				})
				.setRange(0, 100)
				.build()
		);

		// 2
		page.space();
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.guiScale"))
				.setAccessors(() -> options.guiScale, integer -> {
					options.guiScale = integer;
					mc.onResolutionChanged();
				})
				.setFormatter(integer -> new LiteralText(integer == 0 ? "Auto" : integer + "x"))
				.setRange(0, 4)
				.build()
		);
		page.add(new BoolOption.Builder()
				.setName(new TranslatableText("options.fullscreen"))
				.setAccessors(() -> options.fullscreen, aBoolean -> {
					options.fullscreen = aBoolean;

					if (mc.getWindow().isFullscreen() != options.fullscreen) {
						mc.getWindow().toggleFullscreen();
						options.fullscreen = mc.getWindow().isFullscreen();
					}
				})
				.build()
		);
		page.add(new BoolOption.Builder()
				.setName(new TranslatableText("options.vsync"))
				.setAccessors(() -> options.enableVsync, aBoolean -> {
					options.enableVsync = aBoolean;
					mc.getWindow().setVsync(aBoolean);
				})
				.build()
		);
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.framerateLimit"))
				.setAccessors(() -> options.maxFps, integer -> options.maxFps = integer)
				.setFormatter(integer -> integer == 260 ? new TranslatableText("options.framerateLimit.max") : new LiteralText(String.valueOf(integer)))
				.setRange(5, 260)
				.setStep(5)
				.build()
		);

		// 3
		page.space();
		page.add(new BoolOption.Builder()
				.setName(new TranslatableText("options.viewBobbing"))
				.setAccessors(() -> options.bobView, aBoolean -> options.bobView = aBoolean)
				.build()
		);
		page.add(new EnumOption.Builder<AttackIndicator>()
				.setName(new TranslatableText("options.attackIndicator"))
				.setAccessors(() -> options.attackIndicator, attackIndicator -> options.attackIndicator = attackIndicator)
				.setFormatter(attackIndicator -> new TranslatableText(attackIndicator.getTranslationKey()))
				.build()
		);
		page.add(new BoolOption.Builder()
				.setName(new TranslatableText("options.autosaveIndicator"))
				.setAccessors(() -> options.showAutosaveIndicator, aBoolean -> options.showAutosaveIndicator = aBoolean)
				.build()
		);

		return page;
	}

	private Page createElectrum() {
		Page page = new Page(new LiteralText("Electrum"));

		String rustSettings = WgpuNative.getSettings();

		List<Option<?>> options = GSON.fromJson(rustSettings, SETTINGS_TYPE_TOKEN.getType());

//		Map<String, JsonObject> settingsMap = GSON.fromJson(rustSettings,
//				new TypeToken<Map<String, JsonObject>>() {}.getType());
//
//		for (var entry : SETTINGS_STRUCTURE.entrySet()) {
//			var name = entry.getKey();
//			var settingInfo = entry.getValue();
//			var setting = settingsMap.get(name);
//			if (setting == null)
//				return page;
//		}


		return page;
	}

	private Page createQuality() {
		Page page = new Page(new LiteralText("Quality"));

		MinecraftClient mc = MinecraftClient.getInstance();
		GameOptions options = mc.options;

		// 1
		page.add(new EnumOption.Builder<GraphicsMode>()
				.setName(new TranslatableText("options.graphics"))
				.setAccessors(() -> options.graphicsMode, graphicsMode -> {
					options.graphicsMode = graphicsMode;
					mc.worldRenderer.reload();
				})
				.setFormatter(graphicsMode -> new TranslatableText(graphicsMode.getTranslationKey()))
				.build()
		);

		// 2
		page.space();
		page.add(new EnumOption.Builder<CloudRenderMode>()
				.setName(new TranslatableText("options.renderClouds"))
				.setAccessors(() -> options.cloudRenderMode, cloudRenderMode -> options.cloudRenderMode = cloudRenderMode)
				.setFormatter(cloudRenderMode -> new TranslatableText(cloudRenderMode.getTranslationKey()))
				.build()
		);
		page.add(new EnumOption.Builder<ParticlesMode>()
				.setName(new TranslatableText("options.particles"))
				.setAccessors(() -> options.particles, particlesMode -> options.particles = particlesMode)
				.setFormatter(particlesMode -> new TranslatableText(particlesMode.getTranslationKey()))
				.build()
		);
		page.add(new EnumOption.Builder<AoMode>()
				.setName(new TranslatableText("options.ao"))
				.setAccessors(() -> options.ao, aoMode -> {
					options.ao = aoMode;
					mc.worldRenderer.reload();
				})
				.setFormatter(aoMode -> new TranslatableText(aoMode.getTranslationKey()))
				.build()
		);
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.biomeBlendRadius"))
				.setAccessors(() -> options.biomeBlendRadius, integer -> {
					options.biomeBlendRadius = integer;
					mc.worldRenderer.reload();
				})
				.setFormatter(integer -> {
					int i = integer * 2 + 1;
					return new TranslatableText("options.biomeBlendRadius." + i);
				})
				.setRange(0, 7)
				.build()
		);

		// 3
		page.space();
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.entityDistanceScaling"))
				.setAccessors(() -> (int) (options.entityDistanceScaling * 100), integer -> options.entityDistanceScaling = integer / 100f)
				.setFormatter(integer -> new LiteralText(integer + "%"))
				.setRange(50, 500)
				.setStep(25)
				.build()
		);
		page.add(new BoolOption.Builder()
				.setName(new TranslatableText("options.entityShadows"))
				.setAccessors(() -> options.entityShadows, aBoolean -> options.entityShadows = aBoolean)
				.build()
		);

		// 4
		page.space();
		page.add(new IntOption.Builder()
				.setName(new TranslatableText("options.mipmapLevels"))
				.setAccessors(() -> options.mipmapLevels, integer -> options.mipmapLevels = integer)
				.setFormatter(integer -> new LiteralText(integer + "x"))
				.setRange(0, 4)
				.build()
		);

		return page;
	}

	public static class Page implements Iterable<List<Option<?>>> {
		public final Text name;
		private final List<List<Option<?>>> groups = new ArrayList<>();

		public Page(Text name) {
			this.name = name;

			space();
		}

		public void add(Option<?> option) {
			groups.get(groups.size() - 1).add(option);
		}

		public void space() {
			groups.add(new ArrayList<>());
		}

		public boolean isChanged() {
			for (List<Option<?>> group : groups) {
				for (Option<?> option : group) {
					if (option.isChanged()) return true;
				}
			}

			return false;
		}

		public void apply() {
			for (List<Option<?>> group : groups) {
				for (Option<?> option : group) option.apply();
			}
		}

		public void undo() {
			for (List<Option<?>> group : groups) {
				for (Option<?> option : group) option.undo();
			}
		}

		@NotNull
		@Override
		public Iterator<List<Option<?>>> iterator() {
			return groups.iterator();
		}
	}
}
