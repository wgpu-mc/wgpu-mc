package dev.birb.wgpu.rust;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.HashMap;

public class WgpuNative {

    public static ClassLoader getClassLoader() {
        return WgpuNative.class.getClassLoader();
    }

    public static void loadWm() {
        try {
            WgpuNative.load("wgpu_mc_jni", true);
            
//            CoreLib.init();
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
            InputStream is = WgpuNative.class.getClassLoader().getResourceAsStream("assets/wgpu_mc/natives/" + name);
            if (is == null) throw new FileNotFoundException("Could not find lib " + name + " in jar");

            Files.copy(is, object.toPath(), StandardCopyOption.REPLACE_EXISTING);
        }
        System.load(object.getAbsolutePath());
    }

    /**
     * returns  true if the operation succeeded
     */
    public static native int getTextureId(String identifier);

    public static native String getBackend();

    public static native void setAllocator(long ptr);


    public static native void createDevice(long display, long window, int w, int h);



}
