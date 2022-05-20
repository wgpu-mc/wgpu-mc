package eapi.buffer;

import eapi.SpectrumRenderer;

/**
 * Represents a contiguous section of memory on the GPU which can be used for various purposes
 */
public interface SpectrumBuffer {

    int getSize();

    /**
     * {@link SpectrumRenderer#getCapabilities()} must specify that SSBOs are supported for this to succeed.
     * If it doesn't, an error will be thrown.
     *
     * @return {@link SpectrumBindableBuffer}
     * @throws SpectrumRenderer.CapabilityError
     */
    SpectrumBindableBuffer createBindable() throws SpectrumRenderer.CapabilityError;

    EUsage[] getUsages();

    public static enum EUsage {

        SSBO,
        VertexBuffer,
        InstanceBuffer,

    }

}
