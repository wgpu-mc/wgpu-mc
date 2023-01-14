package dev.birb.wgpu.rust;

import net.minecraft.resource.ResourceNotFoundException;
import net.minecraft.util.collection.IndexedIterable;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.HashMap;

public class WgpuNative {

    static {
        loadWm();
    }

    public static void loadWm() {
        try {
            WgpuNative.load("wgpu_mc_jni", true);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    private static final HashMap<Object, Long> idLists = new HashMap<>();

    /**
     * Loads a native library from the resources of this Jar
     *
     * @param name           Library to load
     * @param forceOverwrite Force overwrite the library file
     * @throws ResourceNotFoundException Library not found in resources
     * @throws IOException               Cannot move library out of Jar
     */
    public static void load(String name, boolean forceOverwrite) throws ResourceNotFoundException, IOException {
        name = System.mapLibraryName(name);
        File libDir = new File("lib");
        if (!libDir.exists()) libDir.mkdirs();
        File object = new File("lib", name);
        if (forceOverwrite || !object.exists()) {
            InputStream is = WgpuNative.class.getClassLoader().getResourceAsStream("META-INF/natives/" + name);
            if (is == null) throw new ResourceNotFoundException(object, "Could not find lib " + name + " in jar");

            Files.copy(is, object.toPath(), StandardCopyOption.REPLACE_EXISTING);
        }
        System.load(object.getAbsolutePath());
    }

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

    public static native byte[] digestInputStream(InputStream stream);

    public static native String getBackend();

    public static native void setWorldRenderState(boolean render);

    public static native void texImage2D(int textureId, int target, int level, int internalFormat, int width, int height, int border, int format, int _type, long pixels_ptr);

    public static native void subImage2D(int texId, int target, int level, int offsetX, int offsetY, int width, int height, int format, int _type, long pixels, int unpack_pixel_skip_rows, int unpack_skip_pixels, int unpack_skip_rows, int unpack_alignment);

    public static native void submitCommands();

    public static native int getWindowWidth();

    public static native int getWindowHeight();

    public static native void wmUsePipeline(int i);

    public static native void clearColor(float red, float green, float blue);

    public static native void setIndexBuffer(int[] buffer);

    public static native void debugPalette(long storage, long palette);

    public static native void setVertexBuffer(byte[] buffer);

    public static native void setProjectionMatrix(float[] mat);

    public static native void drawIndexed(int count);

    public static native void draw(int count);

    public static native void attachTextureBindGroup(int slot, int texture);

    public static native double getMouseX();

    public static native double getMouseY();

    public static native void runHelperThread();

    public static native String getVideoMode();

    public static native long createPalette(long idList);

    public static native void destroyPalette(long rustPalettePointer);

    public static native int paletteIndex(long ptr, Object object, int index);

    public static native Object paletteGet(long ptr, int id);

    public static native long copyPalette(long rustPaletteIndex);

    public static native int paletteSize(long rustPalettePointer);

    public static native long createPaletteStorage(long[] copy, int elementsPerLong, int elementBits, long maxValue, int indexScale, int indexOffset, int indexShift, int size);

    public static long uploadIdList(IndexedIterable<Object> idList) {
        if(!idLists.containsKey(idList)) {
            long rustIdList = createIdList();

            idLists.put(idList, rustIdList);

            for(Object entry : idList) {
                int id = idList.getRawId(entry);
                addIdListEntry(rustIdList, id, entry);
            }

            return rustIdList;
        } else {
            return idLists.get(idList);
        }
    }

    private static native long createIdList();

    private static native void addIdListEntry(long idList, int id, Object object);

    public static native void setCursorPosition(double x, double y);

    public static native void setCursorMode(int mode);

    public static native int paletteReadPacket(long rustPalettePointer, byte[] array, int currentPosition, int[] blockstateOffsets);

    public static native void registerBlock(String name);

    public static native void clearPalette(long l);

    public static native void createChunk(int x, int z, long[] pointers, long[] storagePointers);

    public static native void destroyPaletteStorage(long paletteStorage);

    public static native void cacheBlockStates();

    public static native void setCamera(double x, double y, double z, float renderYaw, float renderPitch);

    public static native void bakeChunk(int x, int z);

    public static native int piaGet(long ptr, int x, int y, int z);

    public static native int piaGetByIndex(long ptr, int index);

    public static native void debugBake();

    public static native void setMatrix(int type, float[] mat);

    public static native void setChunkOffset(int x, int z);

    public static native void setCursorLocked(boolean locked);

    public native static void centerCursor();

    public static native void clearChunks();

    public static native void setBlockStateRenderLayer(int rustBlockStateIndex, int layerId);

    public static native void createRenderLayerFilters();

}
