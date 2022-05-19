package eapi.pipeline;

import eapi.EBindable;
import eapi.ELayout;
import eapi.ETexture;

import javax.annotation.Nullable;

public class EPipelineSettings {

    @Nullable
    private final ETexture depth;
    private final ETexture output;

    @Nullable
    private final ELayout<EBindable.Type> bindableTypes;
    private final ELayout<EPipeline.VertexAttributeType> vertexLayout;

    public EPipelineSettings(@Nullable ETexture depth, ETexture output, @Nullable ELayout<EBindable.Type> bindableTypes, ELayout<EPipeline.VertexAttributeType> vertexLayout) {
        this.depth = depth;
        this.output = output;
        this.bindableTypes = bindableTypes;
        this.vertexLayout = vertexLayout;
    }

    @Nullable
    public ETexture getDepth() {
        return depth;
    }

    public ETexture getOutput() {
        return output;
    }

    @Nullable
    public ELayout<EBindable.Type> getBindableTypes() {
        return bindableTypes;
    }

    public ELayout<EPipeline.VertexAttributeType> getVertexLayout() {
        return vertexLayout;
    }
}
