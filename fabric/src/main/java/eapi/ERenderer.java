package eapi;

import eapi.pipeline.EPipeline;
import eapi.pipeline.EPipelineSettings;
import eapi.pipeline.EShader;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.resource.Resource;

public interface ERenderer {

    /**
     * Pipelines should always be re-used. Creating a pipeline should be considered an expensive operation and should only be
     * done during resource loading, and not during rendering. The actual performance impact depends on the implementation of {@link ERenderer}
     * and could be completely negligible, or not.
     *
     * @param name The name of the pipeline, for debugging purposes.
     * @param settings
     * @return The pipeline itself
     */
    EPipeline createPipeline(
            String name,
            EPipelineSettings settings
    );

    Capabilities getCapabilities();

    ETexture uploadTexture(NativeImage image);

    public static interface Capabilities {

        boolean computeShaders();

        boolean SSBOs();

        ETexture.Format[] supportedTextureFormats();

    }

    /**
     * An implementation of {@link ERenderer} *must* support at least GLSL for shaders. Anything else is optional and not required.
     * How GLSL + any other language is supported is an implementation detail.
     *
     * @param vertexSource A resource that when resolved will be a file containg the vertex shader source
     * @param fragmentSource A resource that when resolved will be a file containg the vertex shader source
     * @return {@link EShader} The shader itself, ready to be used in a pipeline
     */
    EShader createShader(String vertexSource, String fragmentSource);

    /**
     * This is thrown by methods that use functionality that the {@link ERenderer} did not specify in it's {@link Capabilities}
     */
    class CapabilityError extends Exception {

    }

}
