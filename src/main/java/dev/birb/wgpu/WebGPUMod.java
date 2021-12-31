package dev.birb.wgpu;

import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.fabricmc.api.ModInitializer;
import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.client.render.block.BlockModels;
import net.minecraft.client.util.ModelIdentifier;
import net.minecraft.resource.ResourceManager;
import net.minecraft.resource.ResourceType;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.profiler.Profiler;
import net.minecraft.util.registry.Registry;

import java.io.IOException;
import java.io.InputStream;
import java.util.HashMap;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executor;

public class WebGPUMod implements ModInitializer {

	@Override
	public void onInitialize() {

//		ClientChunkEvents.CHUNK_LOAD.register((world, chunk) -> {
//			int[] blocks = new int[16 * 16 * 256];
//
//			for(int x = 0; x < 16;x++) {
//				for(int y = 0; y < 256; y++) {
//					for(int z = 0; z < 16;z++) {
//						BlockState state = chunk.getBlockState(new BlockPos(x, y, z));
//						if(state != null) {
//							Identifier id = BlockModels.getModelId(state);
//
//							blocks[(x + (z * 16)) + (y * 16 * 16)] = Wgpu.blocks.get(
//									id.toString()
//							);
//						} else {
//							blocks[(x + (z * 16)) + (y * 64)] = 0;
//						}
//					}
//				}
//			}
//
//			WgpuNative.uploadChunkSimple(blocks, chunk.getPos().x, chunk.getPos().z);
//			System.out.println("Uploaded chunk");
//		});

	}
}
