package dev.birb.wgpu.mixin.render;

import com.llamalad7.mixinextras.sugar.Local;
import dev.birb.wgpu.render.RebuildTaskAccessor;
import net.minecraft.client.render.chunk.ChunkBuilder;
import net.minecraft.client.render.chunk.ChunkRendererRegionBuilder;
import net.minecraft.util.math.BlockPos;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ChunkBuilder.BuiltChunk.class)
public class BuiltChunkMixin {
    @Shadow @Final BlockPos.Mutable origin;

    @Inject(method = "rebuild", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/render/chunk/ChunkBuilder$BuiltChunk$Task;run(Lnet/minecraft/client/render/chunk/BlockBufferBuilderStorage;)Ljava/util/concurrent/CompletableFuture;", shift = At.Shift.BEFORE))
    public void specifyBuiltChunkSync(ChunkRendererRegionBuilder builder, CallbackInfo ci, @Local ChunkBuilder.BuiltChunk.Task task) {
        ((RebuildTaskAccessor) task).wgpu_mc$setBuiltChunk((ChunkBuilder.BuiltChunk) (Object) this);
    }

    @Inject(method = "scheduleRebuild(Lnet/minecraft/client/render/chunk/ChunkBuilder;Lnet/minecraft/client/render/chunk/ChunkRendererRegionBuilder;)V", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/render/chunk/ChunkBuilder;send(Lnet/minecraft/client/render/chunk/ChunkBuilder$BuiltChunk$Task;)V", shift = At.Shift.BEFORE))
    public void specifyBuiltChunk(ChunkBuilder chunkRenderer, ChunkRendererRegionBuilder builder, CallbackInfo ci, @Local ChunkBuilder.BuiltChunk.Task task) {
        ((RebuildTaskAccessor) task).wgpu_mc$setBuiltChunk((ChunkBuilder.BuiltChunk) (Object) this);
    }
//
//    /**
//     * @author wgpu-mc
//     * @reason we do this in Rust
//     */
//    @Overwrite
//    public void scheduleRebuild(ChunkBuilder chunkRenderer, ChunkRendererRegionBuilder builder) {
//        ((ChunkBuilder.BuiltChunk) (Object) this).createRebuildTask(builder);
//    }
//    /**
//     * @author wgpu-mc
//     * @reason Rust builds the chunks
//     */
//    @Inject(method = "createRebuildTask", cancellable = true, at = @At("RETURN"), locals = LocalCapture.CAPTURE_FAILHARD)
//    public void createRebuildTask(ChunkRendererRegionBuilder builder, CallbackInfoReturnable<ChunkBuilder.BuiltChunk.Task> cir) {
////        long[] paletteIndices = new long[27];
////        long[] storageIndices = new long[27];
////        Arrays.fill(paletteIndices,-1);
////        Arrays.fill(storageIndices,-1);
////        ClientWorld world = MinecraftClient.getInstance().world;
////
////        MinecraftClient client = MinecraftClient.getInstance();
////        ChunkLightProvider<?, ?> skyLightProvider = world.getLightingProvider().skyLightProvider;
////        ChunkLightProvider<?, ?> blockLightProvider = world.getLightingProvider().blockLightProvider;
////
////        byte[][] skyIndices = new byte[27][2048];
////        byte[][] blockIndices = new byte[27][2048];
////        Vec3i sectionCoord = new Vec3i(origin.getX()>>4,origin.getY()>>4,origin.getZ()>>4);
////        for(int x=0;x<3;x++){
////            for(int z=0;z<3;z++){
////                WorldChunk chunk = (WorldChunk)world.getChunk(sectionCoord.getX()+x-1, sectionCoord.getZ()+z-1,ChunkStatus.FULL,false);
////                if(chunk==null)continue;
////                for(int y=0;y<3;y++){
////                    int id = x+3*y+9*z;
////                    Palette<?> palette;
////                    PalettedContainer<?> section;
////                    try {
////                        section = chunk.getSection(world.sectionCoordToIndex(sectionCoord.getY()+y-1)).getBlockStateContainer();
////                        palette = section.data.palette;
////                    } catch (ArrayIndexOutOfBoundsException e) {
////                        continue;
////                    }
////
////                    long sectionPos = ChunkSectionPos.from(sectionCoord.getX()+x-1,sectionCoord.getY()+y-1,sectionCoord.getZ()+z-1).asLong();
////                    if(skyLightProvider != null && blockLightProvider != null) {
////                        ChunkNibbleArray skyNibble = skyLightProvider.lightStorage.uncachedStorage.get(sectionPos);
////                        ChunkNibbleArray blockNibble = blockLightProvider.lightStorage.uncachedStorage.get(sectionPos);
////                        if(skyNibble != null) {
////                            skyIndices[id]=skyNibble.asByteArray();
////                        }
////                        if(blockNibble != null) {
////                            blockIndices[id]=blockNibble.asByteArray();
////                        }
////                    }
////
////                    PaletteStorage paletteStorage = section.data.storage;
////
////                    if (paletteStorage instanceof PackedIntegerArray array) {
////                        //palette
////                        RustPalette rustPalette = new RustPalette(section.idList);
////
////                        ByteBuf buf = Unpooled.buffer(palette.getPacketSize());
////                        PacketByteBuf packetBuf = new PacketByteBuf(buf);
////                        if(palette.getSize() == 1){
////                            packetBuf.writeInt(1);
////                        }
////                        palette.writePacket(packetBuf);
////                        rustPalette.readPacket(packetBuf);
////
////                        paletteIndices[id] = rustPalette.getSlabIndex();
////
////                        //PackedIntegerArray
////                        long index = WgpuNative.createPaletteStorage(
////                                paletteStorage.getData(),
////                                array.elementsPerLong,
////                                paletteStorage.getElementBits(),
////                                array.maxValue,
////                                array.indexScale,
////                                array.indexOffset,
////                                array.indexShift,
////                                paletteStorage.getSize()
////                        );
////
////                        storageIndices[id] = index;
////                    }
////                }
////            }
////        }
////        WgpuNative.bakeSection(sectionCoord.getX(),sectionCoord.getY(),sectionCoord.getZ(),paletteIndices, storageIndices, blockIndices, skyIndices);
////        cir.setReturnValue(null);
//    }
//
//    /**
//     * @author wgpu-mc
//     * @reason N/A
//     */
//    @Overwrite
//    public boolean scheduleSort(RenderLayer layer, ChunkBuilder chunkRenderer) {
//        return true;
//    }

}
