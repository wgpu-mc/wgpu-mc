package dev.birb.wgpu;

public class Options {
    public static Backend BACKEND = Backend.Vulkan;
    public static boolean HDR = false;

    public enum Backend {
        Vulkan,
        DirectX12,
        DirectX11,
        Metal,
        Opengl
    }
}
