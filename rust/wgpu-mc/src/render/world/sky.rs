use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyVertex {
    pub position: [f32; 3],
}

impl SkyVertex {
    #[must_use]
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<SkyVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                //Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct SkyboxVertex {
//     pub position: [f32; 3],
//     pub uv: [f32; 2]
// }
//
// impl SkyboxVertex {
//
//     #[must_use]
//     pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<SkyVertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &wgpu::vertex_attr_array![
//                 0 => Float32x3,
//                 1 => Float32x2
//             ]
//         }
//     }
//
// }
