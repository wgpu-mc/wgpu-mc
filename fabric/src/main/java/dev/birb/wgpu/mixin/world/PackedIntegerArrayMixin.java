package dev.birb.wgpu.mixin.world;

import dev.birb.wgpu.palette.PackedIntegerArrayAccessor;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.util.collection.PackedIntegerArray;
import org.apache.commons.lang3.Validate;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
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
        try {
            WgpuNative.load("wgpu_mc_jni", true);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
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
            UNSAFE.putLong(address, m);
        }

        int o = j - l;
        if (o > 0) {
            long p = 0L;

            for(int q = o - 1; q >= 0; --q) {
                p <<= i;
                p |= (long)is[l + q] & this.maxValue;
            }

            long address = this.rawStoragePointer + (k * 8L);
            UNSAFE.putLong(address, p);
        }
    }

    @Override
    public long getStoragePointer() {
        return this.paletteStorage;
    }

    @Inject(method = "swap", at = @At("RETURN"))
    public void swap(int index, int value, CallbackInfoReturnable<Integer> cir) {
        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
        Validate.inclusiveBetween(0L, this.maxValue, (long)value);
        int i = this.getStorageIndex(index);

        long address = this.rawStoragePointer + (((long) i) * 8L);

        long l = UNSAFE.getLong(address);

        int j = (index - i * this.elementsPerLong) * this.elementBits;
        int k = (int)(l >> j & this.maxValue);

        UNSAFE.putLong(address, l & ~(this.maxValue << j) | ((long)value & this.maxValue) << j);

        assert k == cir.getReturnValue();
    }

    @Inject(method = "set", at = @At("RETURN"))
    public void set(int index, int value, CallbackInfo ci) {
        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
        Validate.inclusiveBetween(0L, this.maxValue, (long)value);
        int i = this.getStorageIndex(index);

        long address = this.rawStoragePointer + (((long) i) * 8L);

        long l = UNSAFE.getLong(address);

        int j = (index - i * this.elementsPerLong) * this.elementBits;
        UNSAFE.putLong(address, l & ~(this.maxValue << j) | ((long)value & this.maxValue) << j);
    }

    @Inject(method = "get", at = @At("RETURN"), cancellable = true)
    public void get(int index, CallbackInfoReturnable<Integer> cir) {
        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
        int i = this.getStorageIndex(index);

        long address = this.rawStoragePointer + (((long) i) * 8L);

        long l = UNSAFE.getLong(address);

        int j = (index - i * this.elementsPerLong) * this.elementBits;
        cir.setReturnValue((int)(l >> j & this.maxValue));
    }

    @Inject(method = "method_39892", at = @At("RETURN"))
    public void noClueWhatThisIs(int[] is, CallbackInfo ci) {
        int i = this.data.length;
        int j = 0;

        int k;
        long l;
        int m;
        for(k = 0; k < i - 1; ++k) {
            long address = this.rawStoragePointer + (k * 8L);

            l = UNSAFE.getLong(address);

            for(m = 0; m < this.elementsPerLong; ++m) {
                is[j + m] = (int)(l & this.maxValue);
                l >>= this.elementBits;
            }

            j += this.elementsPerLong;
        }

        k = this.size - j;
        if (k > 0) {
            long address = this.rawStoragePointer + ((i - 1) * 8L);

            l = UNSAFE.getLong(address);

            for(m = 0; m < k; ++m) {
                is[j + m] = (int)(l & this.maxValue);
                l >>= this.elementBits;
            }
        }

    }

}
