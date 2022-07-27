package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.PackedIntegerArrayAccessor;
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

import static dev.birb.wgpu.palette.RustPalette.CLEANER;
import static dev.birb.wgpu.render.Wgpu.UNSAFE;

@Mixin(PackedIntegerArray.class)
public abstract class PackedIntegerArrayMixin implements PackedIntegerArrayAccessor {

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

    @Inject(method = "<init>(II[J)V", at = @At("RETURN"))
    public void createRustStorage(int elementBits, int size, long[] data, CallbackInfo ci) {
        long paletteStorage = WgpuNative.createPaletteStorage(
            this.data,
            this.elementsPerLong,
            this.elementBits,
            this.maxValue,
            this.indexScale,
            this.indexOffset,
            this.indexShift,
            this.size
        );

        this.paletteStorage = paletteStorage;
        this.rawStoragePointer = WgpuNative.getRawStoragePointer(paletteStorage);

        CLEANER.register((PackedIntegerArray) (Object) this, () -> WgpuNative.destroyPaletteStorage(paletteStorage));
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

            long address = this.rawStoragePointer + ((k++) * 8L);
            UNSAFE.putLongVolatile(null, address, m);
        }

        int o = j - l;
        if (o > 0) {
            long p = 0L;

            for(int q = o - 1; q >= 0; --q) {
                p <<= i;
                p |= (long)is[l + q] & this.maxValue;
            }

            long address = this.rawStoragePointer + (k * 8L);
            UNSAFE.putLongVolatile(null, address, p);
        }
    }

    @Override
    public long getStoragePointer() {
        return this.paletteStorage;
    }

    @Overwrite
    // @Inject(method = "swap", at = @At("RETURN"))
    // public void swap(int index, int value, CallbackInfoReturnable<Integer> cir) {
    public int swap(int index, int value) {
        Validate.inclusiveBetween(0L, this.size - 1, index);
        Validate.inclusiveBetween(0L, this.maxValue, value);
        int i = this.getStorageIndex(index);

        long l = UNSAFE.getLongVolatile(null, this.rawStoragePointer + (i * 8L));

        int j = (index - i * this.elementsPerLong) * this.elementBits;
        int k = (int)(l >> j & this.maxValue);

        long data = l & (this.maxValue << j ^ 0xFFFFFFFFFFFFFFFFL) | ((long)value & this.maxValue) << j;
        UNSAFE.putLongVolatile(null, this.rawStoragePointer + (i * 8L), data);

        return k;
        // assert cir.getReturnValue() == k;
    }

    @Overwrite
    // @Inject(method = "set", at = @At("RETURN"))
    // public void set(int index, int value, CallbackInfo ci) {
    public void set(int index, int value) {
        Validate.inclusiveBetween(0L, this.size - 1, index);
        Validate.inclusiveBetween(0L, this.maxValue, value);
        int i = this.getStorageIndex(index);
        long l = UNSAFE.getLongVolatile(null, this.rawStoragePointer + (i * 8L));
        int j = (index - i * this.elementsPerLong) * this.elementBits;
        long data = l & (this.maxValue << j ^ 0xFFFFFFFFFFFFFFFFL) | ((long)value & this.maxValue) << j;

        UNSAFE.putLongVolatile(null, this.rawStoragePointer + (i * 8L), data);
    }

    @Overwrite
    // @Inject(method = "get", at = @At("RETURN"), cancellable = true)
    // public void get(int index, CallbackInfoReturnable<Integer> cir) {
    public int get(int index) {
        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
        int i = this.getStorageIndex(index);

        long address = this.rawStoragePointer + (((long) i) * 8L);

        long l = UNSAFE.getLongVolatile(null, address);

        int j = (index - i * this.elementsPerLong) * this.elementBits;
        int val = (int)(l >> j & this.maxValue);

        // assert val == cir.getReturnValue();
        return val;
    }

    // @Inject(method = "method_39892", at = @At("RETURN"))
    // public void noClueWhatThisIs(int[] is, CallbackInfo ci) {
    @Overwrite
    public void method_39892(int[] is) {
        int i = this.data.length;
        int j = 0;

        int k;
        long l;
        int m;
        for(k = 0; k < i - 1; ++k) {
            long address = this.rawStoragePointer + (k * 8L);

            l = UNSAFE.getLongVolatile(null, address);

            for(m = 0; m < this.elementsPerLong; ++m) {
                is[j + m] = (int)(l & this.maxValue);
                l >>= this.elementBits;
            }

            j += this.elementsPerLong;
        }

        k = this.size - j;
        if (k > 0) {
            long address = this.rawStoragePointer + ((i - 1) * 8L);

            l = UNSAFE.getLongVolatile(null, address);

            for(m = 0; m < k; ++m) {
                is[j + m] = (int)(l & this.maxValue);
                l >>= this.elementBits;
            }
        }

    }

}
