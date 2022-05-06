use std::time::Instant;
use wgpu_mc::mc::block::{BlockState, PackedBlockstateKey};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, CHUNK_VOLUME};
use wgpu_mc::render::world::chunk::BakedChunkLayer;
use wgpu_mc::WmRenderer;

#[derive(Debug)]
struct SimpleBlockstateProvider(PackedBlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> BlockState {
        BlockState {
            packed_key: if x >= 0 && x < 16 && z >= 0 && z < 16 && y < 60 {
                Some(self.0)
            } else {
                None
            }
        }
    }
}

pub fn make_chunks(wm: &WmRenderer) -> Vec<Chunk> {
    let bm = wm.mc.block_manager.read();

    let variant_key = *bm.variant_indices.get(
        "Block{minecraft:blockstates/stone.json}"
    ).unwrap();

    let provider = SimpleBlockstateProvider(variant_key as PackedBlockstateKey);

    let chunk = Chunk::new((0, 0), Box::new(provider));
    let time = Instant::now();

    chunk.bake(&bm);

    println!("Built 1 chunk in {} microseconds", Instant::now().duration_since(time).as_micros());

    vec![chunk]
}