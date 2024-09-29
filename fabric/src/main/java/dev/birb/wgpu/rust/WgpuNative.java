package dev.birb.wgpu.rust;

import net.minecraft.client.MinecraftClient;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.HashMap;

public class WgpuNative {

    static {
        loadWm();
    }

    public static ClassLoader getClassLoader() {
        return WgpuNative.class.getClassLoader();
    }

    public static void loadWm() {
        try {
            WgpuNative.load("wgpu_mc_jni", true);
            WgpuNative.initialize(Thread.currentThread().getContextClassLoader(), MinecraftClient.getInstance());
            
            CoreLib.init();
        } catch (Exception e) {
            throw new IllegalStateException(e);
        }
    }

    private static final HashMap<Object, Long> idLists = new HashMap<>();

    /**
     * Loads a native library from the resources of this Jar
     *
     * @param name           Library to load
     * @param forceOverwrite Force overwrite the library file
     * @throws FileNotFoundException Library not found in resources
     * @throws IOException           Cannot move library out of Jar
     */
    public static void load(String name, boolean forceOverwrite) throws IOException {
        name = System.mapLibraryName(name);
        File libDir = new File("lib");
        if (!libDir.exists()) libDir.mkdirs();
        File object = new File("lib", name);
        if (forceOverwrite || !object.exists()) {
            InputStream is = WgpuNative.class.getClassLoader().getResourceAsStream("META-INF/natives/" + name);
            if (is == null) throw new FileNotFoundException("Could not find lib " + name + " in jar");

            Files.copy(is, object.toPath(), StandardCopyOption.REPLACE_EXISTING);
        }
        System.load(object.getAbsolutePath());
    }

    public static native void initialize(Object object, Object client);
    
    public static native String getSettingsStructure();

    public static native String getSettings();

    /**
     * returns  true if the operation succeeded
     */
    public static native boolean sendSettings(String settings);

    public static native void sendRunDirectory(String dir);

    public static native int getTextureId(String identifier);

    public static native void startRendering(String title);

    public static native void setPanicHook();

    public static native void updateWindowTitle(String title);

    public static native void registerBlockState(Object state, String blockId, String stateKey);

    public static native void doEventLoop();

    public static native String getBackend();

    public static native void setWorldRenderState(boolean render);

    public static native void texImage2D(int textureId, int target, int level, int internalFormat, int width, int height, int border, int format, int type, long pixelsPtr);

    public static native void subImage2D(int texId, int target, int level, int offsetX, int offsetY, int width, int height, int format, int type, int[] pixels, int unpackPixelSkipRows, int unpackSkipPixels, int unpackSkipRows, int unpackAlignment);

    public static native void submitCommands();

    public static native int getWindowWidth();

    public static native int getWindowHeight();

    public static native void wmUsePipeline(int i);

    public static native void clearColor(float red, float green, float blue);

    public static native void setIndexBuffer(int[] buffer);

    public static native void setVertexBuffer(byte[] buffer);

    public static native void setProjectionMatrix(float[] mat);

    public static native void drawIndexed(int count);

    public static native void draw(int count);

    public static native void attachTextureBindGroup(int slot, int texture);

    public static native double getMouseX();

    public static native double getMouseY();

    public static native void runHelperThread();

    public static native String getVideoMode();

    public static native long createPalette();

    public static native void destroyPalette(long rustPalettePointer);

    public static native int paletteIndex(long ptr, Object object, int index);

    public static native int paletteSize(long rustPalettePointer);

    public static native long createPaletteStorage(long[] copy, int elementsPerLong, int elementBits, long maxValue, int indexScale, int indexOffset, int indexShift, int size);

    public static native void setCursorPosition(double x, double y);

    public static native void setCursorMode(int mode);

    public static native int paletteReadPacket(long slabIndex, byte[] array, int currentPosition, long[] blockstateOffsets);

    public static native void registerBlock(String name);

    public static native void clearPalette(long l);

    public static native void destroyPaletteStorage(long paletteStorage);

    public static native void cacheBlockStates();

    public static native void setCamera(double x, double y, double z, float renderYaw, float renderPitch);

    public static native void bakeSection(int x, int y, int z, long[] paletteIndices, long[] storageIndices, byte[][] blockIndices, byte[][] skyIndices);

    public static native void setMatrix(int type, float[] mat);

    public static native void setCursorLocked(boolean locked);

    public static native void centerCursor();

    public static native void registerEntities(String toString);

    public static native long setEntityInstanceBuffer(String entity, long mat4Ptr, int position, long overlayPtr, int overlayArrayPosition, int instanceCount, int textureId);

    public static native void clearEntities();

    public static native void identifyGlTexture(int texture, int glId);

    public static native void scheduleStop();

    public static native long createAndDeserializeLightData(byte[] array, int index);

    public static native void bindLightData(long lightData, int x, int z);

    public static native void setLightmapID(int id);

    public static native void debugLight(int x, int y, int z);

    public static native void setAllocator(long ptr);

    public static native void bindSkyData(float colorR, float colorG, float colorB, float skyPosition, float skyBrightness, float starShimmer, int moonPhase);

    public static native void bindStarData(int length, int[] indices, byte[] vertices);

    public static native void bindRenderEffectsData(float fogStart, float fogEnd, int fogShape, float[] fogColor, float[] colorModulator, float[] dimensionFogColor);

    public static native void reloadStorage(int clampedViewDistance,int minSectionHeight);

    public static native void reloadShaders();

    public static native void setSectionPos(int x,int z);

    public static native void render(float tickDelta, long startTime, boolean tick);

    public static native void setShaderColor(float r, float g, float b, float a);
}
