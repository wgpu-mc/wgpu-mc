package eapi.pipeline;

import eapi.SpectrumBindable;
import eapi.SpectrumLayout;
import eapi.SpectrumTexture;

import javax.annotation.Nullable;
import java.util.List;

public class SpectrumPipelineSettings {

    @Nullable
    private final SpectrumTexture depth;
    private final SpectrumTexture output;

    @Nullable
    private final SpectrumLayout<SpectrumBindable.Type> bindableTypes;
    private final List<SpectrumLayout<SpectrumPipeline.VertexAttributeType>> vertexLayout;

    /**
     * @param depth If this pipeline should use a depth buffer. If this is set to null, fragments from this pipeline will always render.
     * @param output Where the result of this pipelines fragment shader will be rendered to
     * @param bindableTypes A layout describing what type of {@link SpectrumBindable}s will be used during draw calls
     * @param vertexLayout A layout describing the vertex attributes that will be passed into the vertex shader
     */
    public SpectrumPipelineSettings(@Nullable SpectrumTexture depth, SpectrumTexture output, @Nullable SpectrumLayout<SpectrumBindable.Type> bindableTypes, List<SpectrumLayout<SpectrumPipeline.VertexAttributeType>> vertexLayout) {
        this.depth = depth;
        this.output = output;
        this.bindableTypes = bindableTypes;
        this.vertexLayout = List.copyOf(vertexLayout);
    }

    @Nullable
    public SpectrumTexture getDepth() {
        return depth;
    }

    public SpectrumTexture getOutput() {
        return output;
    }

    @Nullable
    public SpectrumLayout<SpectrumBindable.Type> getBindableTypes() {
        return bindableTypes;
    }

    public List<SpectrumLayout<SpectrumPipeline.VertexAttributeType>> getVertexLayout() {
        return vertexLayout;
    }

}
