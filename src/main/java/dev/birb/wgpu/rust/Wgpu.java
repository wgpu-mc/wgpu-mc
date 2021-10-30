package dev.birb.wgpu.rust;

//import net.minecraft.util.Identifier;

public class Wgpu {

    public static native void initialize(String title);

    public static native void updateWindowTitle(String title);

    public static native void registerEntry(int type, String name);

    public static native void doEventLoop();

//    public static native void registerTexture(Identifier identifier);

}
