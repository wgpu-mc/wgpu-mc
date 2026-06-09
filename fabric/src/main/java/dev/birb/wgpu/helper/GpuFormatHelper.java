package dev.birb.wgpu.helper;

import com.mojang.blaze3d.GpuFormat;
import dev.birb.wm.WM;

public class GpuFormatHelper {
    
    public static int gpuFormatToRustEnum(GpuFormat format) {
        return switch(format) {
            case R8_UNORM -> WM.R8_UNORM();
            case R8_SNORM -> WM.R8_SNORM();
            case RG8_UNORM -> WM.RG8_UNORM();
            case RG8_SNORM -> WM.RG8_SNORM();
            case RGB8_UNORM -> WM.RGB8_UNORM();
            case RGB8_SNORM -> WM.RGB8_SNORM();
            case RGBA8_UNORM -> WM.RGBA8_UNORM();
            case RGBA8_SNORM -> WM.RGBA8_SNORM();
            case R16_UNORM -> WM.R16_UNORM();
            case R16_SNORM -> WM.R16_SNORM();
            case RG16_UNORM -> WM.RG16_UNORM();
            case RG16_SNORM -> WM.RG16_SNORM();
            case RGB16_UNORM -> WM.RGB16_UNORM();
            case RGB16_SNORM -> WM.RGB16_SNORM();
            case RGBA16_UNORM -> WM.RGBA16_UNORM();
            case RGBA16_SNORM -> WM.RGBA16_SNORM();
            case R8_UINT -> WM.R8_UINT();
            case R8_SINT -> WM.R8_SINT();
            case RG8_UINT -> WM.RG8_UINT();
            case RG8_SINT -> WM.RG8_SINT();
            case RGB8_UINT -> WM.RGB8_UINT();
            case RGB8_SINT -> WM.RGB8_SINT();
            case RGBA8_UINT -> WM.RGBA8_UINT();
            case RGBA8_SINT -> WM.RGBA8_SINT();
            case R16_UINT -> WM.R16_UINT();
            case R16_SINT -> WM.R16_SINT();
            case RG16_UINT -> WM.RG16_UINT();
            case RG16_SINT -> WM.RG16_SINT();
            case RGB16_UINT -> WM.RGB16_UINT();
            case RGB16_SINT -> WM.RGB16_SINT();
            case RGBA16_UINT -> WM.RGBA16_UINT();
            case RGBA16_SINT -> WM.RGBA16_SINT();
            case R32_UINT -> WM.R32_UINT();
            case R32_SINT -> WM.R32_SINT();
            case RG32_UINT -> WM.RG32_UINT();
            case RG32_SINT -> WM.RG32_SINT();
            case RGB32_UINT -> WM.RGB32_UINT();
            case RGB32_SINT -> WM.RGB32_SINT();
            case RGBA32_UINT -> WM.RGBA32_UINT();
            case RGBA32_SINT -> WM.RGBA32_SINT();
            case R16_FLOAT -> WM.R16_FLOAT();
            case RG16_FLOAT -> WM.RG16_FLOAT();
            case RGB16_FLOAT -> WM.RGB16_FLOAT();
            case RGBA16_FLOAT -> WM.RGBA16_FLOAT();
            case R32_FLOAT -> WM.R32_FLOAT();
            case RG32_FLOAT -> WM.RG32_FLOAT();
            case RGB32_FLOAT -> WM.RGB32_FLOAT();
            case RGBA32_FLOAT -> WM.RGBA32_FLOAT();
            case RGB10A2_UNORM -> WM.RGB10A2_UNORM();
            case RGB10A2_UINT -> WM.RGB10A2_UINT();
            case RG11B10_FLOAT -> WM.RG11B10_FLOAT();
            case D32_FLOAT -> WM.D32_FLOAT();
            case D32_FLOAT_S8_UINT -> WM.D32_FLOAT_S8_UINT();
            case D24_UNORM_S8_UINT -> WM.D24_UNORM_S8_UINT();
            case D16_UNORM -> WM.D16_UNORM();
            case S8_UINT -> WM.S8_UINT();
        };

    }
    
}
