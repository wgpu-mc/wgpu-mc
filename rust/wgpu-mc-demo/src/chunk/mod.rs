use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk};
use wgpu_mc::mc::MinecraftState;
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::terrain::BLOCK_ATLAS_NAME;
use wgpu_mc::WmRenderer;

struct SimpleBlockstateProvider(Arc<MinecraftState>, BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        if y == 0 && x.abs_diff(7) < 8 && z.abs_diff(7) < 8 {
            ChunkBlockState::State(self.1)
        } else {
            ChunkBlockState::Air
        }
    }
}

impl Debug for SimpleBlockstateProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

pub fn make_chunks(wm: &WmRenderer) -> Chunk {
    let bm = wm.mc.block_manager.read();
    let atlas = wm
        .mc
        .texture_manager
        .atlases
        .load()
        .get(BLOCK_ATLAS_NAME)
        .unwrap()
        .load();

    let (index, _, fence) = bm.blocks.get_full("minecraft:oak_fence").unwrap();
    let fence_model = fence
        .get_model_by_key(
            [("north", &StateValue::Bool(true))],
            &*wm.mc.resource_provider,
            &atlas,
        )
        .unwrap();

    println!("{:?}", fence_model);

    let provider = SimpleBlockstateProvider(
        wm.mc.clone(),
        BlockstateKey {
            block: index as u16,
            augment: fence_model.1,
        },
    );

    let chunk = Chunk::new((0, 0));
    let time = Instant::now();

    chunk.bake(&bm, &provider);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );

    chunk
}
