package eapi;

import eapi.buffer.SpectrumBuffer;
import eapi.pipeline.SpectrumPipeline;
import eapi.pipeline.SpectrumPipelineSettings;
import eapi.pipeline.SpectrumShader;
import net.minecraft.client.texture.NativeImage;

import java.nio.ByteBuffer;

public interface SpectrumRenderer {

    /**
     * Pipelines should always be re-used. Creating a pipeline should be considered an expensive operation and should only be
     * done during resource loading, and not during rendering. The actual performance impact depends on the implementation of {@link SpectrumRenderer}
     * and could be completely negligible, or not.
     *
     * @param name The name of the pipeline, for debugging purposes.
     * @param settings The pipeline settings
     * @return The pipeline itself
     */
    SpectrumPipeline createPipeline(
            String name,
            SpectrumPipelineSettings settings
    );

    Capabilities getCapabilities();

    SpectrumTexture createTexture(NativeImage image);

    SpectrumBuffer createBuffer(ByteBuffer bytes);

    public static interface Capabilities {

        boolean computeShaders();

        boolean SSBOs();

        SpectrumTexture.Format[] supportedTextureFormats();

    }

    /**
     * An implementation of {@link SpectrumRenderer} *must* support at least GLSL for shaders. Anything else is optional and not required.
     * How GLSL + any other language is supported is an implementation detail.
     *
     * @param vertexSource A resource that when resolved will be a file containg the vertex shader source
     * @param fragmentSource A resource that when resolved will be a file containg the vertex shader source
     * @return {@link SpectrumShader} The shader itself, ready to be used in a pipeline
     */
    SpectrumShader createShader(String vertexSource, String fragmentSource);

    /**
     * This is thrown by methods that use functionality that the {@link SpectrumRenderer} did not specify in it's {@link Capabilities}
     */
    class CapabilityError extends Exception {

    }

}
