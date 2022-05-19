package eapi;


import eapi.pipeline.EPipeline;

public interface EBindable {

    /**
     * This is mostly only used when creating a new {@link EPipeline}, and during draw call validation in the aforementioned pipelines
     */
    public static enum Type {

        Buffer,
        Texture

    }

    Type getType();

}
