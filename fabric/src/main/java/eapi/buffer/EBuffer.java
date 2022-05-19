package eapi.buffer;

import eapi.ERenderer;

public interface EBuffer {

    int getSize();

    /**
     * {@link ERenderer#getCapabilities()} must specify that SSBOs are supported for this to succeed.
     * If it doesn't, an error will be thrown.
     *
     * @return {@link EBindableBuffer}
     * @throws ERenderer.CapabilityError
     */
    EBindableBuffer createBindable() throws ERenderer.CapabilityError;

    EUsage[] getUsages();

    public static enum EUsage {

        SSBO,
        VertexBuffer,
        InstanceBuffer,

    }

}
