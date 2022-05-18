package eapi;

import net.minecraft.client.texture.NativeImage;

public interface ERenderer {

    EPipeline createPipeline(
            String name,
            EBindable.Type[] bindables,
            EVertexAttributeType[] vertexAttributeTypes
    );

    ERendererCapabilities[] getCapabilities();

    ETexture uploadTexture(NativeImage image);

    public static enum ERendererCapabilities {
        Compute,
        SSBO
    }

}
