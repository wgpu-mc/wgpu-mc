use std::time::Instant;
use wgpu_mc::mc::block::{ChunkBlockState, BlockstateKey};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, CHUNK_VOLUME};
use wgpu_mc::render::world::chunk::BakedChunkLayer;
use wgpu_mc::WmRenderer;

#[derive(Debug)]
struct SimpleBlockstateProvider(BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        ChunkBlockState {
            packed_key: if x >= 0 && x < 16 && z >= 0 && z < 16 {
                Some(self.0)
            } else {
                None
            },
        }
    }
}

pub fn make_chunks(wm: &WmRenderer) -> Vec<Chunk> {
    let bm = wm.mc.block_manager.read();

    let variant_key: BlockstateKey = (*bm
        .block_key_indices
        .get("Block{minecraft:blockstates/cobblestone.json}")
        .unwrap() as u32).into();

    let provider = SimpleBlockstateProvider(variant_key);

    let chunk = Chunk::new((0, 0), Box::new(provider));
    let time = Instant::now();

    chunk.bake(&bm);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );

    vec![chunk]
}
