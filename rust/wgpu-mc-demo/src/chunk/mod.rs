use std::collections::HashMap;
use std::time::Instant;
use wgpu_mc::mc::block::{ChunkBlockState, BlockstateKey};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, CHUNK_VOLUME};
use wgpu_mc::render::world::chunk::BakedChunkLayer;
use wgpu_mc::WmRenderer;

#[derive(Debug)]
struct SimpleBlockstateProvider(BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        if x >= 0 && x < 16 && z >= 0 && z < 16 {
            ChunkBlockState::State(self.0)
        } else {
            ChunkBlockState::Air
        }
    }
}

pub fn make_chunks(wm: &WmRenderer) -> Vec<Chunk> {
    let mut bm = wm.mc.block_manager.write();

    let magma_block_key = BlockstateKey {
        block: bm.blocks.get_full("minecraft:magma_block").unwrap().0 as u16,
        augment: 0,
    };

    let provider = SimpleBlockstateProvider(magma_block_key);

    let chunk = Chunk::new((0, 0));
    let time = Instant::now();

    chunk.bake(&bm, &provider);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );

    vec![chunk]
}
