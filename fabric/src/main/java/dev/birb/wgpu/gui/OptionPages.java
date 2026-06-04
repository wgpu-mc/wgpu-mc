package dev.birb.wgpu.gui;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import dev.birb.wgpu.gui.options.*;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.AttackIndicatorStatus;
import net.minecraft.client.GraphicsPreset;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Options;
import net.minecraft.network.chat.Component;
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
        Page page = new Page(Component.of("General"));

        Minecraft mc = Minecraft.getInstance();
        Options options = mc.options;

        // 1
        page.add(new IntOption.Builder()
                .setName(Component.translatable("options.renderDistance"))
                .setOption(options.renderDistance())
                .setFormatter(integer -> Component.translatable("options.chunks", integer))
                .setRange(2, 32)
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Component.translatable("options.simulationDistance"))
                .setOption(options.simulationDistance())
                .setFormatter(integer -> Component.translatable("options.chunks", integer))
                .setRange(5, 16)
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Component.translatable("options.gamma"))
                .setAccessors(() -> (int) (options.gamma().get() * 100), integer -> options.gamma().set(integer / 100.0))
                .setFormatter(integer -> {
                    if (integer == 0) return Component.translatable("options.gamma.min");
                    else if (integer == 50) return Component.translatable("options.gamma.default");
                    else if (integer == 100) return Component.translatable("options.gamma.max");

                    return Component.literal(integer + "%");
                })
                .setRange(0, 100)
                .build()
        );


        // 2
        page.space();
        page.add(new IntOption.Builder()
                .setName(Component.translatable("options.guiScale"))
                .setOption(options.guiScale(), _ -> {})
                .setFormatter(integer -> Component.literal(integer == 0 ? "Auto" : integer + "x"))
                .setRange(0, 4)
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Component.translatable("options.fullscreen"))
                .setOption(options.fullscreen())
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Component.translatable("options.vsync"))
                .setOption(options.enableVsync())
                .build()
        );
        page.add(new IntOption.Builder()
                .setName(Component.translatable("options.framerateLimit"))
                .setOption(options.framerateLimit())
                .setFormatter(integer -> integer == 260 ? Component.translatable("options.framerateLimit.max") : Component.literal(String.valueOf(integer)))
                .setRange(5, 260)
                .setStep(5)
                .build()
        );

        // 3
        page.space();
        page.add(new BoolOption.Builder()
                .setName(Component.translatable("options.viewBobbing"))
                .setOption(options.bobView())
                .build()
        );
        page.add(new EnumOption.Builder<>(AttackIndicatorStatus.class)
                .setName(Component.translatable("options.attackIndicator"))
                .setOption(options.attackIndicator())
                .setFormatter(attackIndicator -> Component.translatable(attackIndicator.name()))
                .build()
        );
        page.add(new BoolOption.Builder()
                .setName(Component.translatable("options.autosaveIndicator"))
                .setOption(options.showAutosaveIndicator())
                .build()
        );

        return page;
    }

    private Page createElectrum() {
        Page page = new Page(Component.literal("Electrum"));

        String rustSettings = WgpuNative.getSettings();

        List<Option<?>> options = GSON.fromJson(rustSettings, SETTINGS_TYPE_TOKEN.getType());

        for (var option : options) {
            page.add(option);
        }

        return page;
    }

    private Page createQuality() {
        Page page = new Page(Component.literal("Quality"));

        Minecraft mc = Minecraft.getInstance();
        Options options = mc.options;

        // 1
        page.add(new EnumOption.Builder<>(GraphicsPreset.class)
                .setName(Component.translatable("options.graphics"))
                .setOption(options.graphicsPreset())
                .setFormatter(graphicsMode -> Component.translatable(graphicsMode.getKey()))
                .build()
        );

        // 2
//        page.space();
//        page.add(new EnumOption.Builder<>(CloudRenderMode.class)
//                .setName(Component.translatable("options.renderClouds"))
//                .setOption(options.getCloudRenderMode())
//                .setFormatter(cloudRenderMode -> Component.translatable(cloudRenderMode.name()))
//                .build()
//        );
//        page.add(new EnumOption.Builder<>(ParticlesMode.class)
//                .setName(Component.translatable("options.particles"))
//                .setOption(options.getParticles())
//                .setFormatter(particlesMode -> Component.translatable(particlesMode.getTranslationKey()))
//                .build()
//        );
//        page.add(new BoolOption.Builder()
//                .setName(Component.translatable("options.ao"))
//                .setOption(options.getAo())
//                .build()
//        );
//        page.add(new IntOption.Builder()
//                .setName(Component.translatable("options.biomeBlendRadius"))
//                .setOption(options.getBiomeBlendRadius())
//                .setFormatter(integer -> {
//                    int i = integer * 2 + 1;
//                    return Component.translatable("options.biomeBlendRadius." + i);
//                })
//                .setRange(0, 7)
//                .build()
//        );

        // 3
//        page.space();
//        page.add(new IntOption.Builder()
//                .setName(Component.translatable("options.entityDistanceScaling"))
//                .setAccessors(() -> (int) (options.getEntityDistanceScaling().getValue() * 100), integer -> options.getEntityDistanceScaling().setValue(integer / 100.0))
//                .setFormatter(integer -> Component.of(integer + "%"))
//                .setRange(50, 500)
//                .setStep(25)
//                .build()
//        );
//        page.add(new BoolOption.Builder()
//                .setName(Component.translatable("options.entityShadows"))
//                .setOption(options.getEntityShadows())
//                .build()
//        );
//
//        // 4
//        page.space();
//        page.add(new IntOption.Builder()
//                .setName(Component.translatable("options.mipmapLevels"))
//                .setOption(options.getMipmapLevels())
//                .setFormatter(integer -> Component.of(integer + "x"))
//                .setRange(0, 4)
//                .build()
//        );

        return page;
    }

    public static class Page implements Iterable<List<Option<?>>> {
        public final Component name;
        private final List<List<Option<?>>> groups = new ArrayList<>();

        public Page(Component name) {
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
