package dev.birb.wgpu.backend;

import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;

public class WgpuCompiledRenderPipeline implements CompiledRenderPipeline {

    @Override
    public boolean isValid() {
        return true;
    }

}
