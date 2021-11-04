package dev.birb.wgpu.rust;

//import net.minecraft.util.Identifier;

//import net.minecraft.world.chunk.ChunkSection;

import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.WorldChunk;

public class Wgpu {

    public static native void initialize(String title);

    public static native void updateWindowTitle(String title);

    public static native void registerEntry(int type, String name);

    public static native void doEventLoop();

    public static native void uploadChunk(WorldChunk chunk);

    public static native void registerSprite(String namespace);

    public static native String getBackend();

//    public static native void registerTexture(Identifier identifier);

}
