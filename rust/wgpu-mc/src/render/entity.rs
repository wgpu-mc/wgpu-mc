#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EntityVertex {
    pub position: [f32; 3],
    pub tex_coords: [u16; 2],
    pub normal: [f32; 3],
    pub part_id: u32,
}

impl EntityVertex {
    const VAA: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Uint32,
        2 => Float32x3,
        3 => Uint32
    ];

    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<EntityVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VAA,
        }
    }
}
