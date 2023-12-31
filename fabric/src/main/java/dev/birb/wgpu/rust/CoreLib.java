package dev.birb.wgpu.rust;

import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;

public class CoreLib {
    
    public static void init() {
        CoreLib.initAllocator(MemoryUtil.getAllocator());
//        CoreLib.initPanicHandler();
    }

//    private static void initPanicHandler() {
//        CoreLibFFI.setPanicHandler(CALLBACK.address());
//    }

    private static void initAllocator(MemoryUtil.MemoryAllocator allocator) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            PointerBuffer pfn = stack.mallocPointer(4);
            pfn.put(0 /* aligned_alloc */, allocator.getAlignedAlloc());
            pfn.put(1 /* aligned_free */, allocator.getAlignedFree());
            pfn.put(2 /* realloc */, allocator.getRealloc());
            pfn.put(3 /* calloc */, allocator.getCalloc());

            WgpuNative.setAllocator(pfn.address());
        }
    }

}
