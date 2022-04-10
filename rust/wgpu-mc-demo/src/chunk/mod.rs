use wgpu_mc::mc::block::BlockState;
use wgpu_mc::mc::chunk::{Chunk, CHUNK_VOLUME};
use wgpu_mc::render::world::chunk::BakedChunkLayer;
use wgpu_mc::WmRenderer;

pub fn make_chunk(wm: &WmRenderer) -> Chunk {
    let bm = wm.mc.block_manager.read();

    let variant_key = *bm.variant_indices.get(
        &nsr!("minecraft:blockstates/anvil.json#facing=north")
    ).unwrap();

    println!("{:?}", bm.block_state_variants.get(variant_key).unwrap());

    let mut chunk = Chunk::new(
        (0, 0),
        Box::new([BlockState {
            packed_key: Some(variant_key as u32)
        }; CHUNK_VOLUME])
    );

    chunk
}