use wgpu_mc::mc::block::BlockState;
use wgpu_mc::mc::chunk::{Chunk, CHUNK_VOLUME};
use wgpu_mc::render::world::chunk::BakedChunkLayer;
use wgpu_mc::WmRenderer;

// pub fn make_chunks(wm: &WmRenderer) -> Vec<Chunk> {
//     let bm = wm.mc.block_manager.read();
//
//     let variant_key = *bm.variant_indices.get(
//         &nsr!("minecraft:blockstates/diamond_ore.json#")
//     ).unwrap();
//
//     (0..1).map(|index| Chunk::new(
//         (index % 10, index / 10),
//         Box::new([BlockState {
//             packed_key: Some(variant_key as u32)
//         }; CHUNK_VOLUME])
//     )).collect()
// }