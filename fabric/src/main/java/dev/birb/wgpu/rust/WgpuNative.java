package dev.birb.wgpu.rust;

//import net.minecraft.util.Identifier;

//import net.minecraft.world.chunk.ChunkSection;

import net.minecraft.resource.ResourceNotFoundException;
import net.minecraft.world.chunk.WorldChunk;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.HashMap;

public class WgpuNative {

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

    public static native int getTextureId(String identifier);

    public static native void startRendering(String title);

    public static native void preInit();

    public static native void updateWindowTitle(String title);

    public static native void registerEntry(int type, String name);

    public static native void doEventLoop();

    public static native byte[] digestInputStream(InputStream stream);

    public static native String getBackend();

    public static native HashMap<String, Integer> bakeBlockModels();

    public static native void setWorldRenderState(boolean render);

    public static native void texImage2D(int textureId, int target, int level, int internalFormat, int width, int height, int border, int format, int _type, long pixels_ptr);

    public static native void subImage2D(int texId, int target, int level, int offsetX, int offsetY, int width, int height, int format, int _type, long pixels, int unpack_pixel_skip_rows, int unpack_skip_pixels, int unpack_skip_rows, int unpack_alignment);

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

    public static native void scheduleChunkRebuild(int x, int z);

    public static native void setCursorPosition(double x, double y);

    public static native void setCursorMode(int mode);
}
