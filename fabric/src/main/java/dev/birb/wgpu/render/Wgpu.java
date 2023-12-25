package dev.birb.wgpu.render;

import dev.birb.wgpu.entity.EntityState;
import dev.birb.wgpu.palette.RustBlockStateAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import dev.birb.wgpu.rust.WgpuTextureManager;
import lombok.Getter;
import lombok.Setter;
import net.minecraft.client.MinecraftClient;
import sun.misc.Unsafe;

import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.HashMap;

import static dev.birb.wgpu.WgpuMcMod.LOGGER;

public class Wgpu {
    @Getter
    private static final WgpuTextureManager textureManager = new WgpuTextureManager();

    @Getter
    @Setter
    private static volatile boolean initialized = false;

    @Getter
    @Setter
    private static volatile boolean mayInitialize = false;

    public static HashMap<String, Integer> blocks;
    public static WgpuTextureManager getTextureManager() {
        return textureManager;
    }
    public static HashMap<Integer, Integer> keyStates = new HashMap<>();
    public static ArrayList<Runnable> injectPartIds = new ArrayList<>();

    @Getter
    private static RuntimeException exception;

    @Getter
    @Setter
    private static String wmIdentity;

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
        client.execute(() -> client.keyboard.onChar(0, codepoint, modifiers));
    }

    @SuppressWarnings("unused") // called from rust
    public static void keyState(int key, int scancode, int state, int modifiers) {
        MinecraftClient client = MinecraftClient.getInstance();
        Wgpu.keyStates.put(key, state);

        client.execute(() -> client.keyboard.onKey(0, key, scancode, state, modifiers));
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
        RuntimeException exception = new RuntimeException(message);
        LOGGER.error(message);
        while (true) {
            // wait for main loop to catch this
        }
    }

    @SuppressWarnings("unused") // called from rust
    public static void rustDebug(String message) {
        LOGGER.info("[Engine] " + message);
    }

    public static void helperSetBlockStateIndex(Object o, int blockstateKey) {
        ((RustBlockStateAccessor) o).wgpu_mc$setRustBlockStateIndex(blockstateKey);
    }

    public static void helperSetPartIndex(String entity, String part, int index) {


        if(!EntityState.matrixIndices.containsKey(entity)) {
            EntityState.matrixIndices.put(entity, new HashMap<>());
        }

        EntityState.matrixIndices.get(entity).put(part, index);
    }

    @SuppressWarnings("unused") // called from rust
    public static void windowFocused(boolean focused) {
        MinecraftClient.getInstance().onWindowFocusChanged(focused);
    }

    public static void incrementTexSubImageCount() {
        timesTexSubImageCalled++;
    }

}
