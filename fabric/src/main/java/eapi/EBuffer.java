package eapi;

public interface EBuffer {

    public static enum EUsages {

        //ERenderer implementation must support SSBOs
        Bindable,
        RenderAttachment,
        VertexBuffer,
        InstanceBuffer,

    }

}
