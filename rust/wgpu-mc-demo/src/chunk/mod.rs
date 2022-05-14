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
    let mut bm = wm.mc.block_manager.write();

    let variant_key = *bm
        .block_state_indices
        .get("Block{minecraft:blockstates/bamboo.json}")
        .unwrap() as BlockstateKey;

    let anvil_key = *bm
        .block_state_indices
        .get("Block{minecraft:blockstates/anvil.json}[facing=north]")
        .unwrap() as BlockstateKey;

    let cobblestone_key = *bm
        .block_state_indices
        .get("Block{minecraft:blockstates/cobblestone.json}")
        .unwrap() as BlockstateKey;

    let command_block_key = *bm
        .block_state_indices
        .get("Block{minecraft:blockstates/chain_command_block.json}[conditional=false,facing=north]")
        .unwrap() as BlockstateKey;

    let magma_block_key = *bm
        .block_state_indices
        .get("Block{minecraft:blockstates/magma_block.json}")
        .unwrap() as BlockstateKey;

    let block_variant = bm.block_states.get(variant_key as usize).unwrap().1.clone();

    let mut map = HashMap::new();
    map.insert("age".into(), "0".into());

    bm.block_states.push(
        (
            map,
            block_variant
        )
    );

    let key = (bm.block_states.len() - 1) as BlockstateKey;

    let provider = SimpleBlockstateProvider(command_block_key);

    let chunk = Chunk::new((0, 0), Box::new(provider));
    let time = Instant::now();

    chunk.bake(&bm);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );

    vec![chunk]
}
