package dev.birb.wgpu.mixin.world;

import net.minecraft.util.collection.PackedIntegerArray;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import java.nio.LongBuffer;

@Mixin(PackedIntegerArray.class)
public abstract class PackedIntegerArrayMixin {

    @Shadow protected abstract int getStorageIndex(int index);

    @Shadow @Final public int elementsPerLong;
    @Shadow @Final private int elementBits;
    @Shadow @Final public long maxValue;
    @Shadow @Final private int size;
    private LongBuffer data;

//    @Inject(method = "<init>(II)V", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/collection/PackedIntegerArray;<init>(II[J)V", shift = At.Shift.AFTER))
//    private void initIIv(int elementBits, int size, CallbackInfo ci) {
//
//    }
//
//    @Inject(method = "<init>(II[J)V", at = @At(value = "INVOKE", target = "Ljava/lang/Object;<init>()V", shift = At.Shift.AFTER))
//    private void init_(int elementBits, int size, long[] data, CallbackInfo ci) {
//
//    }
//
//    /**
//     * @author wgpu-mc
//     * @reason Use direct byte buffers
//     */
//    @Overwrite
//    public int swap(int index, int value) {
//        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
//        Validate.inclusiveBetween(0L, this.maxValue, (long)value);
//        int i = this.getStorageIndex(index);
//        long l = this.data.get(i);
//        int j = (index - i * this.elementsPerLong) * this.elementBits;
//        int k = (int)(l >> j & this.maxValue);
//        this.data.put(i, l & ~(this.maxValue << j) | ((long)value & this.maxValue) << j);
//        return k;
//    }
//
//    /**
//     * @author
//     * @reason
//     */
//    @Overwrite
//    public void set(int index, int value) {
//        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
//        Validate.inclusiveBetween(0L, this.maxValue, (long)value);
//        int i = this.getStorageIndex(index);
//        long l = this.data.get(i);
//        int j = (index - i * this.elementsPerLong) * this.elementBits;
//        this.data.put(i, l & ~(this.maxValue << j) | ((long)value & this.maxValue) << j);
//    }
//
//    /**
//     * @author
//     * @reason
//     */
//    @Overwrite
//    public int get(int index) {
//        Validate.inclusiveBetween(0L, (long)(this.size - 1), (long)index);
//        int i = this.getStorageIndex(index);
//        long l = this.data.get(i);
//        int j = (index - i * this.elementsPerLong) * this.elementBits;
//        return (int)(l >> j & this.maxValue);
//    }
//
//    @Overwrite
//    public void forEach(IntConsumer action) {
//        int i = 0;
//
//        for (long l : this.data.) {
//            for (int j = 0; j < this.elementsPerLong; j++) {
//                action.accept((int)(l & this.maxValue));
//                l >>= this.elementBits;
//                if (++i >= this.size) {
//                    return;
//                }
//            }
//        }
//    }

}
