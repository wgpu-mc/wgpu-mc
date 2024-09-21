use std::fmt::Debug;
use std::time::Instant;

use glam::{ivec3, IVec3};
use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{bake_section, BlockStateProvider, LightLevel};
use wgpu_mc::mc::{Scene};
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::WmRenderer;
struct SimpleBlockstateProvider(BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, pos: IVec3) -> ChunkBlockState {
        if ((pos.x / 2 & 1 == 0) ^ (pos.z / 2 & 1 == 0) ^ (pos.y & 1 == 0) && (pos.y == 0 || pos.y == 1) && pos.y < 2) || pos.y == 5 {
        // if pos.x ^ pos.y ^ pos.z == 0 {
        // if pos.y == 0 {
            ChunkBlockState::State(self.0)
        } else {
            ChunkBlockState::Air
        }
    }

    fn get_light_level(&self, _pos: IVec3) -> LightLevel {
        // let dist = _pos.rem_euclid(ivec3(32, 32, 32)).distance_squared(ivec3(16, 16, 16)) / 2;
        // let value = (16 - dist) as u8;
        
        let value = (_pos.x as f32 / 8.0).sin().abs() * 15.0;
        
        LightLevel::from_sky_and_block(value as u8, value as u8)
    }

    fn is_section_empty(&self, _relpos: IVec3) -> bool {
        false
    }
}

impl Debug for SimpleBlockstateProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

pub fn make_chunks(wm: &WmRenderer, pos: IVec3, scene: &Scene) {
    let bm = wm.mc.block_manager.read();
    let atlases = wm.mc.texture_manager.atlases.read();
    let atlas = atlases.get(BLOCK_ATLAS).unwrap();

    let (index, _, block) = bm.blocks.get_full("minecraft:quartz_block").unwrap();

    let (_, augment) = block
        .get_model_by_key(
            // [("facing", &StateValue::String("north".into()))],
            [],
            &*wm.mc.resource_provider,
            atlas,
            0,
        )
        .unwrap();

    let provider = SimpleBlockstateProvider(BlockstateKey {
        block: index as u16,
        augment,
    });
    let time = Instant::now();
    bake_section(pos, wm, &provider);
}
