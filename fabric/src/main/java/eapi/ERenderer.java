package eapi;

import eapi.pipeline.EPipeline;
import eapi.pipeline.EPipelineSettings;
import net.minecraft.client.texture.NativeImage;

public interface ERenderer {

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
     * This is thrown by methods that use functionality that the {@link ERenderer} did not specify in it's {@link Capabilities}
     */
    class CapabilityError extends Exception {

    }

}
