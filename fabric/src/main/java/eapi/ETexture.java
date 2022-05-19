package eapi;

import eapi.pipeline.EPipeline;
import net.minecraft.client.texture.NativeImage;

/**
 * Represents a texture which has been uploaded to the GPU. All textures are automatically usable as an {@link EBindable} in shaders
 */
public interface ETexture extends EBindable {

    int getWidth();

    int getHeight();

    Format getFormat();

    void upload(NativeImage image) throws TextureSizeError;

    /**
     * Defines texture formats that a given {@link ETexture} can use. Any color format which has a per-color bit-depth higher than 8
     * may not be supported by a given {@link ERenderer}
     */
    enum Format {
        Depth,
        Rgba8,
        Bgra8,
        Rgba16,
        Rgba10,
        Rgba12
    }

    /**
     * {@link Usage} is a specifier for extra functionality that this {@link ETexture} can be used for.
     * {@link Usage#PipelineOutput} means that this texture can be used as the output texture of an {@link EPipeline}
     */
    enum Usage {
        PipelineOutput
    }

    /**
     * This will be thrown in two situations: <br>
     *     - The {@link ERenderer} does not support a texture of this size OR <br>
     *     - An attempt was made to upload new data to an already existing texture, where there is a dimension and/or format mismatch between the upload attempt and the existing image on the GPU
     */
    class TextureSizeError extends Error {

    }

}
