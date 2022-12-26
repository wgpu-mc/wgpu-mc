use bytemuck::{Pod, Zeroable};

#[derive(Zeroable, Pod, Copy, Clone)]
#[repr(C)]
pub struct ChunkOffset {
    start: u32,
    len: u32,
    x: f32,
    z: f32,
}
