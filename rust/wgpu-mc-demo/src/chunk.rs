use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use glam::IVec3;
use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{bake_section, BlockStateProvider, LightLevel, Section};
use wgpu_mc::mc::{MinecraftState, Scene};
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::WmRenderer;
struct SimpleBlockstateProvider(BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self,pos:IVec3) -> ChunkBlockState {
        // if (0..1).contains(&x) && (0..1).contains(&z) && y == 0 {
        if pos.x^pos.y^pos.z == 0 {
            ChunkBlockState::State(self.0)
        } else {
            ChunkBlockState::Air
        }
    }

    fn get_light_level(&self,_pos:IVec3) -> LightLevel {
        LightLevel::from_sky_and_block(15, 15)
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

pub fn make_chunks(wm: &WmRenderer, pos: IVec3, scene: &Scene){
    let bm = wm.mc.block_manager.read();
    let atlases = wm
        .mc
        .texture_manager
        .atlases.read();
    let atlas = atlases
        .get(BLOCK_ATLAS)
        .unwrap();

    let (index, _, block) = bm.blocks.get_full("minecraft:wall_torch").unwrap();

    let (_, augment) = block
        .get_model_by_key(
            [
            ("facing", &StateValue::String("north".into())),
            ],
            &*wm.mc.resource_provider,
            &atlas,
            0,
        )
        .unwrap();

    let provider = SimpleBlockstateProvider(
        BlockstateKey {
            block: index as u16,
            augment,
        },
    );
    let time = Instant::now();
    bake_section(pos, wm, &provider);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );
}
