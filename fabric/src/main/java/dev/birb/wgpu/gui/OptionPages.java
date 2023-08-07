package dev.birb.wgpu.gui;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import dev.birb.wgpu.gui.options.*;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.*;
import net.minecraft.text.Text;
import org.jetbrains.annotations.NotNull;

import java.util.*;

public class OptionPages implements Iterable<OptionPages.Page> {
    private static final TypeToken<Map<String, RustOptionInfo>> SETTINGS_STRUCTURE_TYPE_TOKEN = new TypeToken<>() {
    };
    private static final TypeToken<List<Option<?>>> SETTINGS_TYPE_TOKEN = new TypeToken<>() {
    };
    private static final Gson GSON = new GsonBuilder()
            .registerTypeAdapter(SETTINGS_TYPE_TOKEN.getType(), new Option.OptionSerializerDeserializer())
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
        Page page = new Page(Text.of("General"));

        MinecraftClient mc = MinecraftClient.getInstance();
        GameOptions options = mc.options;

        // 1
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.renderDistance"))
                .setOption(options.getViewDistance())
                .setFormatter(integer -> Text.translatable("options.chunks", integer))
                .setRange(2, 32)
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.simulationDistance"))
                .setOption(options.getSimulationDistance())
                .setFormatter(integer -> Text.translatable("options.chunks", integer))
                .setRange(5, 16)
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.gamma"))
                .setAccessors(() -> (int) (options.getGamma().getValue() * 100), integer -> options.getGamma().setValue(integer / 100.0))
                .setFormatter(integer -> {
                    if (integer == 0) return Text.translatable("options.gamma.min");
                    else if (integer == 50) return Text.translatable("options.gamma.default");
                    else if (integer == 100) return Text.translatable("options.gamma.max");

                    return Text.of(integer + "%");
                })
                .setRange(0, 100)
                .build()
        );

        // 2
        page.space();
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.guiScale"))
                .setOption(options.getGuiScale(), i -> mc.onResolutionChanged())
                .setFormatter(integer -> Text.of(integer == 0 ? "Auto" : integer + "x"))
                .setRange(0, 4)
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Text.translatable("options.fullscreen"))
                .setOption(options.getFullscreen())
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Text.translatable("options.vsync"))
                .setOption(options.getEnableVsync())
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.framerateLimit"))
                .setOption(options.getMaxFps())
                .setFormatter(integer -> integer == 260 ? Text.translatable("options.framerateLimit.max") : Text.of(String.valueOf(integer)))
                .setRange(5, 260)
                .setStep(5)
                .build()
        );

        // 3
        page.space();
        page.add(new BoolOption.Builder()
                .setName(Text.translatable("options.viewBobbing"))
                .setOption(options.getBobView())
                .build()
        );
        page.add(new EnumOption.Builder<>(AttackIndicator.class)
                .setName(Text.translatable("options.attackIndicator"))
                .setOption(options.getAttackIndicator())
                .setFormatter(attackIndicator -> Text.translatable(attackIndicator.getTranslationKey()))
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Text.translatable("options.autosaveIndicator"))
                .setOption(options.getShowAutosaveIndicator())
                .build()
        );

        return page;
    }

    private Page createElectrum() {
        Page page = new Page(Text.of("Electrum"));

        String rustSettings = WgpuNative.getSettings();

        List<Option<?>> options = GSON.fromJson(rustSettings, SETTINGS_TYPE_TOKEN.getType());

        for (var option : options) {
            page.add(option);
        }

        return page;
    }

    private Page createQuality() {
        Page page = new Page(Text.of("Quality"));

        MinecraftClient mc = MinecraftClient.getInstance();
        GameOptions options = mc.options;

        // 1
        page.add(new EnumOption.Builder<>(GraphicsMode.class)
                .setName(Text.translatable("options.graphics"))
                .setOption(options.getGraphicsMode())
                .setFormatter(graphicsMode -> Text.translatable(graphicsMode.getTranslationKey()))
                .build()
        );

        // 2
        page.space();
        page.add(new EnumOption.Builder<>(CloudRenderMode.class)
                .setName(Text.translatable("options.renderClouds"))
                .setOption(options.getCloudRenderMode())
                .setFormatter(cloudRenderMode -> Text.translatable(cloudRenderMode.getTranslationKey()))
                .build()
        );
        page.add(new EnumOption.Builder<>(ParticlesMode.class)
                .setName(Text.translatable("options.particles"))
                .setOption(options.getParticles())
                .setFormatter(particlesMode -> Text.translatable(particlesMode.getTranslationKey()))
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Text.translatable("options.ao"))
                .setOption(options.getAo())
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.biomeBlendRadius"))
                .setOption(options.getBiomeBlendRadius())
                .setFormatter(integer -> {
                    int i = integer * 2 + 1;
                    return Text.translatable("options.biomeBlendRadius." + i);
                })
                .setRange(0, 7)
                .build()
        );

        // 3
        page.space();
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.entityDistanceScaling"))
                .setAccessors(() -> (int) (options.getEntityDistanceScaling().getValue() * 100), integer -> options.getEntityDistanceScaling().setValue(integer / 100.0))
                .setFormatter(integer -> Text.of(integer + "%"))
                .setRange(50, 500)
                .setStep(25)
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Text.translatable("options.entityShadows"))
                .setOption(options.getEntityShadows())
                .build()
        );

        // 4
        page.space();
        page.add(new IntOption.Builder()
                .setName(Text.translatable("options.mipmapLevels"))
                .setOption(options.getMipmapLevels())
                .setFormatter(integer -> Text.of(integer + "x"))
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
            if (Objects.equals(this.name.getString(), "Electrum")) {
                var options = groups.stream().flatMap(Collection::stream).toList();
                var json = GSON.toJson(options, SETTINGS_TYPE_TOKEN.getType());
                WgpuNative.sendSettings(json);
            } else {
                for (List<Option<?>> group : groups) {
                    for (Option<?> option : group) option.apply();
                }

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
