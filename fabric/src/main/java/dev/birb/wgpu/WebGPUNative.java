package dev.birb.wgpu;

import net.minecraft.resource.ResourceNotFoundException;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;

public class WebGPUNative {
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
            InputStream is = WebGPUNative.class.getClassLoader().getResourceAsStream("META-INF/natives/" + name);
            if (is == null) throw new ResourceNotFoundException(object, "Could not find lib " + name + " in jar");

            Files.copy(is, object.toPath(), StandardCopyOption.REPLACE_EXISTING);
        }
        System.load(object.getAbsolutePath());
    }
}
