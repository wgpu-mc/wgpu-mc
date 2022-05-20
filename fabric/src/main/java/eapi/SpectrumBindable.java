package eapi;


import eapi.pipeline.SpectrumPipeline;

public interface SpectrumBindable {

    /**
     * This is mostly only used when creating a new {@link SpectrumPipeline}, and during draw call validation in the aforementioned pipelines
     */
    public static enum Type {

        Buffer,
        Texture

    }

    Type getType();

}
