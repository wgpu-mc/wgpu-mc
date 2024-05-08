use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use glam::IVec3;
use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{BlockStateProvider, LightLevel, Section};
use wgpu_mc::mc::{MinecraftState, Scene};
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::WmRenderer;
struct SimpleBlockstateProvider(Arc<MinecraftState>, BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i32, z: i32) -> ChunkBlockState {
        // if (0..1).contains(&x) && (0..1).contains(&z) && y == 0 {
        if y == 0 {
            ChunkBlockState::State(self.1)
        } else {
            ChunkBlockState::Air
        }
    }

    fn get_light_level(&self, _x: i32, _y: i32, _z: i32) -> LightLevel {
        LightLevel::from_sky_and_block(15, 15)
    }

    fn is_section_empty(&self, _index: usize) -> bool {
        false
    }
}

impl Debug for SimpleBlockstateProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

pub fn make_chunks(wm: &WmRenderer, pos: IVec3, scene: &Scene) -> Section {
    let bm = wm.mc.block_manager.read();
    let atlas = wm
        .mc
        .texture_manager
        .atlases
        .load()
        .get(BLOCK_ATLAS)
        .unwrap()
        .load();

    let (index, _, block) = bm.blocks.get_full("minecraft:furnace").unwrap();

    let (_, augment) = block
        .get_model_by_key(
            [
                ("facing", &StateValue::String("north".into())),
                ("lit", &StateValue::Bool(true)),
            ],
            &*wm.mc.resource_provider,
            &atlas,
            0,
        )
        .unwrap();

    let provider = SimpleBlockstateProvider(
        wm.mc.clone(),
        BlockstateKey {
            block: index as u16,
            augment,
        },
    );

    let mut chunk = Section::new(pos.into());
    let time = Instant::now();

    chunk.bake_section(wm, &bm, &mut scene.chunk_allocator.lock(), scene.chunk_buffer.buffer.clone(), &provider);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );

    chunk
}
