use wgpu_mc::mc::block::{BlockState, PackedBlockstateKey};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, CHUNK_VOLUME};
use wgpu_mc::render::world::chunk::BakedChunkLayer;
use wgpu_mc::WmRenderer;

#[derive(Debug)]
struct SimpleBlockstateProvider(PackedBlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> BlockState {
        BlockState {
            packed_key: Some(self.0)
        }
    }
}

pub fn make_chunks(wm: &WmRenderer) -> Vec<Chunk> {
    let bm = wm.mc.block_manager.read();

    let variant_key = *bm.variant_indices.get(
        "Block{minecraft:blockstates/diamond_ore.json}"
    ).unwrap();

    let provider = SimpleBlockstateProvider(variant_key as PackedBlockstateKey);

    let chunk = Chunk::new((0, 0), Box::new(provider));
    chunk.bake(&bm);

    vec![chunk]
}