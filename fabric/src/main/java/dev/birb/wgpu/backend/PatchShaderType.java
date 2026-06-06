package dev.birb.wgpu.backend;

import com.mojang.blaze3d.shaders.ShaderType;

public enum PatchShaderType {
    VERTEX(ShaderType.VERTEX, ".vsh"),
    FRAGMENT(ShaderType.FRAGMENT, ".fsh");

    public final ShaderType glShaderType;
    public final String extension;

    PatchShaderType(ShaderType glShaderType, String extension) {
        this.glShaderType = glShaderType;
        this.extension = extension;
    }

    public static PatchShaderType[] fromGlShaderType(ShaderType glShaderType) {
        return switch (glShaderType) {
            case VERTEX -> new PatchShaderType[]{VERTEX};
            case FRAGMENT -> new PatchShaderType[]{FRAGMENT};
            default -> throw new IllegalArgumentException("Unknown shader type: " + glShaderType);
        };
    }
}