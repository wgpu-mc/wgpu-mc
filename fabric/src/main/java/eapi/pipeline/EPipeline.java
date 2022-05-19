package eapi.pipeline;

import eapi.EBindable;
import eapi.ELayout;
import eapi.buffer.EBuffer;

public interface EPipeline {

    void draw(int vertices, int instances, EBindable[] bindings, EBuffer[] vertexBuffers);

    record VertexLayout(ELayout<VertexAttribute> vertexAttributes) {}

    record VertexAttribute(VertexAttributeType type, int location) {}

    enum VertexAttributeType {

        Float_x1,
        Float_x2,
        Float_x3,

        Int

    }

}
