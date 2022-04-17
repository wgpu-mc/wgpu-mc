package dev.birb.wgpu.mixin.world;

import org.spongepowered.asm.mixin.Mixin;

@Mixin(targets = { "net.minecraft.client.world.ClientChunkManager$ClientChunkMap" })
public class ClientChunkMapMixin {

    private static int uploaded_chunks = 0;

//    @Inject(method = "set", at = @At("HEAD"))
//    protected void set(int index, WorldChunk chunk, CallbackInfo ci) {
//
//        if(uploaded_chunks > 10) {
//            return;
//        }
//
//        int[] blocks = new int[16 * 16 * 256];
//
//        for(int x = 0; x < 16;x++) {
//            for(int y = 0; y < 256; y++) {
//                for(int z = 0; z < 16;z++) {
//                    BlockState state = chunk.getBlockState(new BlockPos(x + chunk.getPos().getStartX(), y, z+chunk.getPos().getStartZ()));
//                    if(state != null) {
//                        Identifier id = BlockModels.getModelId(state);
//                        if(y < 50 && x == 1) {
//                            System.out.println(id);
//                        }
////                        try {
////                            blocks[(x + (z * 16)) + (y * 64)] = WebGPUMod.blockIds.get(new Identifier(
////                                    id.getNamespace(),
////                                    "blockstates/" + id.getPath() + ".json"
////                            ).toString());
////                        } catch(Exception e) {
////                            System.out.println("Null block");
////                            blocks[(x + (z * 16)) + (y * 64)] = 0;
////                        }
//                    } else {
//                        blocks[(x + (z * 16)) + (y * 64)] = 0;
//                    }
//                }
//            }
//        }
//
////        long time = System.currentTimeMillis();
//        System.out.println("Uploading chunk");
//        WgpuNative.uploadChunkSimple(blocks, chunk.getPos().x, chunk.getPos().z);
//        System.out.println("Done");
////        System.out.println(System.currentTimeMillis() - time);
//
//        uploaded_chunks++;
//    }

}
