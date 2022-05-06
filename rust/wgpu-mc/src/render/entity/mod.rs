use crate::model::BindableTexture;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EntityVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub part_id: u32,
}

impl EntityVertex {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<EntityVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                //Texcoords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                //Normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                //Part
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
/// Data to describe an instance of an entity type on the GPU
pub struct EntityRenderInstance {
    /// Index into mat4[]
    pub entity_index: u32,
    /// Index into the float2[] to describe the offset of this entities texture
    pub entity_texture_index: u32,
    pub parts_per_entity: u32,
}

impl EntityRenderInstance {
    const VAA: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        4 => Uint32,
        5 => Uint32,
        6 => Uint32
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<EntityRenderInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VAA,
        }
    }
}

pub struct EntityGroupInstancingFrame {
    ///The model for the entity
    pub vbo: Arc<wgpu::Buffer>,
    ///`EntityRenderInstance`s
    pub instance_vbo: Arc<wgpu::Buffer>,

    ///mat4x4<f32>[] for model-part transforms
    pub part_transform_matrices: Arc<wgpu::BindGroup>,
    ///vec2<f32>[] for offsets for mob variant textures
    pub texture_offsets: Arc<wgpu::BindGroup>,
    ///the texture
    pub texture: Arc<BindableTexture>,
    ///how many entities to draw
    pub instance_count: u32,
    ///how many vertices for this kind of entity
    pub vertex_count: u32,
}
