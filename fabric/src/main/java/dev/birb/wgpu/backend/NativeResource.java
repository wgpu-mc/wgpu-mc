package dev.birb.wgpu.backend;

import java.lang.foreign.MemorySegment;

public interface NativeResource {

    MemorySegment getNativeUnsafe();

    boolean isClosed();

    default MemorySegment getNative() {
        if(this.isClosed()) throw new IllegalStateException("Attempting to access closed resource");
        return this.getNativeUnsafe();
    }

}
