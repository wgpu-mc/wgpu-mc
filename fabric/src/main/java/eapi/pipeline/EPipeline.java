package eapi.pipeline;

import eapi.EBindable;
import eapi.ELayout;
import eapi.buffer.EBuffer;

import java.util.List;

public interface EPipeline {

    /**
     *
     * @param vertices The amount of vertices to be rendered. Must be 0 or greater.
     * @param instances The amount of instances to be rendered. Must be 0 or greater.
     * @param bindings An array of {@link EBindable}s which will be used in this draw call. The entries of this array must match the types specified in the {@link EPipelineSettings}
     * @param vertexBuffers An array of {@link EBuffer}s which contain the vertex data.
     * @throws DrawCallError This will be thrown if <br>
     * - the vertex or instance count is less than zero, <br>
     * - the bindings do not match the pipeline settings <br>
     * - there are no specified vertex buffers, or there are not enough vertex buffers in correspondence to the pipeline settings <br>
     * - this method is called outside of the game's render loop (there is no active render pass)
     */
    void draw(int vertices, int instances, EBindable[] bindings, EBuffer[] vertexBuffers) throws DrawCallError;

    /**
     * This should be considered an expensive operation and should only be done during resource (re-)loading
     * @param shader The shader that this pipeline should use
     */
    void setShader(EShader shader);

    EShader getShader();

    EPipelineSettings getSettings();

    record VertexLayout(List<ELayout<VertexAttribute>> vertexAttributes) {}

    record VertexAttribute(VertexAttributeType type, int location) {}

    enum VertexAttributeType {

        Float_x1,
        Float_x2,
        Float_x3,

        Int

    }

    class DrawCallError extends Throwable {
    }

}
