package eapi;

public interface ETexture extends EBindableCandidate {

    int getWidth();

    int getHeight();

    ETextureFormat getFormat();

    public static enum ETextureFormat {
        DepthFloat,
        Rgba8,
        Bgra8
    }

}
