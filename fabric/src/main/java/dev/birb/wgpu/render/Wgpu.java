package dev.birb.wgpu.render;

import dev.birb.wgpu.palette.RustBlockStateAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import lombok.Getter;
import lombok.Setter;
import net.minecraft.client.MinecraftClient;
import sun.misc.Unsafe;

import java.lang.reflect.Field;
import java.util.HashMap;
import java.util.Map;

import static dev.birb.wgpu.WgpuMcMod.LOGGER;
import static dev.birb.wgpu.input.WgpuKeys.convertModifiers;

public class Wgpu {
    @Getter
    private static final WgpuTextureManager textureManager = new WgpuTextureManager();

    @Getter
    @Setter
    private static volatile boolean initialized = false;
    @Getter
    @Setter
    private static volatile boolean mayInitialize = false;

    @Getter
    private static RuntimeException exception;

    @Getter
    @Setter
    private static String wmIdentity;

    @Getter
    private static final Map<Integer, Integer> keyStates = new HashMap<>();

    @Getter
    private static int timesTexSubImageCalled = 0;

    @Getter
    private static Unsafe unsafe;

    @Getter
    private static int windowWidth = 1280;
    @Getter
    private static int windowHeight = 720;

    static {
        try {
            Field f = Unsafe.class.getDeclaredField("theUnsafe");
            f.setAccessible(true);
            unsafe = (Unsafe) f.get(null);
        } catch (Exception e) {
            LOGGER.error("Could not get the unsafe", e);
        }
    }

    public static void linkRenderDoc() {
        try {
            System.loadLibrary("renderdoc");
        } catch (UnsatisfiedLinkError e) {
            LOGGER.warn("Error while loading RenderDoc", e);
        }
    }

    public static void startRendering() {
        if (!initialized) {
            linkRenderDoc();
            WgpuNative.startRendering("Minecraft");
        } else {
            throw new IllegalStateException("wgpu-mc has already been initialized");
        }
    }

    @SuppressWarnings("unused") // called from rust
    public static void cursorMove(double x, double y) {
        MinecraftClient client = net.minecraft.client.MinecraftClient.getInstance();

        client.execute(() -> client.mouse.onCursorPos(0, x, y));
    }

    @SuppressWarnings("unused") // called from rust
    public static void mouseMove(double x, double y) {
    }

    @SuppressWarnings("unused") // called from rust
    public static void mouseAction(int button, int action) {
        MinecraftClient client = MinecraftClient.getInstance();

        client.execute(() -> client.mouse.onMouseButton(-1, button, action, 0));
    }

    @SuppressWarnings("unused") // called from rust
    public static void onChar(int codepoint, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        int mappedModifier = convertModifiers(modifiers);
        client.execute(() -> client.keyboard.onChar(0, codepoint, mappedModifier));
    }

    @SuppressWarnings("unused") // called from rust
    public static void keyState(int key, int scancode, int state, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        int convertedModifier = convertModifiers(modifiers);
        Wgpu.keyStates.put(key, state);

        client.execute(() -> client.keyboard.onKey(0, key, scancode, state, convertedModifier));
    }

    @SuppressWarnings("unused") // called from rust
    public static void onResize(int width, int height) {
        Wgpu.windowWidth = width;
        Wgpu.windowHeight = height;
        MinecraftClient client = MinecraftClient.getInstance();
        client.execute(client::onResolutionChanged);
    }

    @SuppressWarnings("unused") // called from rust
    public static void rustPanic(String message) {
        exception = new RuntimeException(message);
        LOGGER.error(message);
        while (true) {
            // wait for main loop to catch this
        }
    }

    @SuppressWarnings("unused") // called from rust
    public static void helperSetBlockStateIndex(Object o, int blockstateKey) {
        ((RustBlockStateAccessor) o).wgpu_mc$setRustBlockStateIndex(blockstateKey);
    }

    @SuppressWarnings("unused") // called from rust
    public static void debug(Object o) {
        LOGGER.info("{}", o);
    }

    @SuppressWarnings("unused") // called from rust
    public static void windowFocused(boolean focused) {
        MinecraftClient.getInstance().onWindowFocusChanged(focused);
    }

    public static void incrementTexSubImageCount() {
        timesTexSubImageCalled++;
    }
}
