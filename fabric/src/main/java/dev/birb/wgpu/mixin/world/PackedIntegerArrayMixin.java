package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.util.collection.PackedIntegerArray;
import org.apache.commons.lang3.Validate;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.function.IntConsumer;

import static dev.birb.wgpu.palette.RustPalette.CLEANER;
import static dev.birb.wgpu.render.Wgpu.UNSAFE;

@Mixin(PackedIntegerArray.class)
public abstract class PackedIntegerArrayMixin {

    static {
        WgpuNative.loadWm();
    }

    @Shadow @Final public int elementsPerLong;
    @Shadow @Final private int elementBits;
    @Shadow @Final public long maxValue;
    @Shadow @Final public int indexScale;
    @Shadow @Final public int indexOffset;
    @Shadow @Final public int indexShift;
    @Shadow @Final private int size;

    @Shadow @Final private long[] data;

    @Shadow protected abstract int getStorageIndex(int index);

    private long rawStoragePointer;
    private long paletteStorage;
    private long maxAddress;

    private LongBuffer buffer;

    @Inject(method = "<init>(II[J)V", at = @At("RETURN"))
    public void createRustStorage(int elementBits, int size, long[] data, CallbackInfo ci) {
        // long paletteStorage = WgpuNative.createPaletteStorage(
        //     this.data,
        //     this.elementsPerLong,
        //     this.elementBits,
        //     this.maxValue,
        //     this.indexScale,
        //     this.indexOffset,
        //     this.indexShift,
        //     this.size
        // );

        ByteBuffer buffer = ByteBuffer.allocateDirect(this.data.length * 8);
        LongBuffer longBuffer = buffer.asLongBuffer();
        longBuffer.put(this.data);
        this.buffer = longBuffer;

        // this.paletteStorage = paletteStorage;
        // this.rawStoragePointer = WgpuNative.getRawStoragePointer(paletteStorage);
        // this.maxAddress = this.rawStoragePointer + (this.data.length * 8L);

        // CLEANER.register((PackedIntegerArray) (Object) this, () -> WgpuNative.destroyPaletteStorage(paletteStorage));
    }

    @Inject(method = "<init>(II[I)V", at = @At("RETURN"))
    public void doTheOtherInit(int i, int j, int[] is, CallbackInfo ci) {
        int k = 0;

        int l;
        for(l = 0; l <= j - this.elementsPerLong; l += this.elementsPerLong) {
            long m = 0L;

            for(int n = this.elementsPerLong - 1; n >= 0; --n) {
                m <<= i;
                m |= (long)is[l + n] & this.maxValue;
            }

            // long address = this.rawStoragePointer + ((k++) * 8L);
            // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);
            // UNSAFE.putLongVolatile(null, address, m);
            this.buffer.put(k++, m);
        }

        int o = j - l;
        if (o > 0) {
            long p = 0L;

            for(int q = o - 1; q >= 0; --q) {
                p <<= i;
                p |= (long)is[l + q] & this.maxValue;
            }

            // long address = this.rawStoragePointer + (k * 8L);
            // long address = (this.rawStoragePointer / 8) + k;
            // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);
            // UNSAFE.putLongVolatile(null, address, p);
            this.buffer.put(k, p);
        }
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    // @Inject(method = "swap", at = @At("RETURN"))
    // public void swap(int index, int value, CallbackInfoReturnable<Integer> cir) {
    public int swap(int index, int value) {
        Validate.inclusiveBetween(0L, this.size - 1, index);
        Validate.inclusiveBetween(0L, this.maxValue, value);
        int i = this.getStorageIndex(index);

        // long address = this.rawStoragePointer + (i * 8L);
        // long address = (this.rawStoragePointer / 8) + i;

        // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);

        // long l = UNSAFE.getLongVolatile(null, address);
        long l = this.buffer.get(i);

        int j = (index - i * this.elementsPerLong) * this.elementBits;
        int k = (int)(l >> j & this.maxValue);

        long data = l & (this.maxValue << j ^ 0xFFFFFFFFFFFFFFFFL) | ((long)value & this.maxValue) << j;
        // UNSAFE.putLongVolatile(null, address, data);
        this.buffer.put(i, data);

        return k;
        // assert cir.getReturnValue() == k;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void set(int index, int value) {
        Validate.inclusiveBetween(0L, this.size - 1, index);
        Validate.inclusiveBetween(0L, this.maxValue, value);
        int i = this.getStorageIndex(index);

        // long address = this.rawStoragePointer + (i * 8L);
        // long address = (this.rawStoragePointer / 8) + i;
        // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);

        // long l = UNSAFE.getLongVolatile(null, address);
        long l = this.buffer.get(i);
        int j = (index - i * this.elementsPerLong) * this.elementBits;
        long data = l & (this.maxValue << j ^ 0xFFFFFFFFFFFFFFFFL) | ((long)value & this.maxValue) << j;

        // UNSAFE.putLongVolatile(null, address, data);
        this.buffer.put(i, data);
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public int get(int index) {
        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);

        int i = this.getStorageIndex(index);

        // long address = this.rawStoragePointer + (((long) i) * 8L);

        // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);

        // long l = UNSAFE.getLongVolatile(null, address);
        long l = this.buffer.get(i);

        // System.out.println(index + " " + address + " " + l);
        int j = (index - i * this.elementsPerLong) * this.elementBits;
        int val = (int)(l >> j & this.maxValue);

        // assert val == cir.getReturnValue();
        return val;
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void forEach(IntConsumer action) {
        int i = 0;
        for(int offset=0;offset<this.data.length;offset++) {
            // long address = this.rawStoragePointer  + (offset * 8L);
            // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);
            // long l = UNSAFE.getLongVolatile(null, address);
            long l = this.buffer.get(offset);

            for (int j = 0; j < this.elementsPerLong; ++j) {
                action.accept((int)(l & this.maxValue));
                l >>= this.elementBits;
                if (++i < this.size) continue;
                return;
            }
        }
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public long[] getData() {
        // throw new RuntimeException("no");
        // return WgpuNative.copyPaletteStorageData(this.paletteStorage);
        LongBuffer copyBuffer = LongBuffer.allocate(this.buffer.capacity());
        copyBuffer.put(this.buffer);
        return copyBuffer.array();
    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void method_39892(int[] is) {
        int i = this.data.length;
        int j = 0;

        int k;
        long l;
        int m;
        for(k = 0; k < i - 1; ++k) {
            // long address = this.rawStoragePointer + (k * 8L);

            // Validate.exclusiveBetween(this.rawStoragePointer, this.maxAddress, address);
            // l = UNSAFE.getLongVolatile(null, address);
            l = this.buffer.get(k);

            for(m = 0; m < this.elementsPerLong; ++m) {
                is[j + m] = (int)(l & this.maxValue);
                l >>= this.elementBits;
            }

            j += this.elementsPerLong;
        }

        k = this.size - j;
        if (k > 0) {
            // long address = this.rawStoragePointer + ((i - 1) * 8L);

            // l = UNSAFE.getLongVolatile(null, address);
            l = this.buffer.get(i - 1);
            

            for(m = 0; m < k; ++m) {
                is[j + m] = (int)(l & this.maxValue);
                l >>= this.elementBits;
            }
        }

    }

}
