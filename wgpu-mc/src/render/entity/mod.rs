use cgmath::Matrix4;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use crate::WmRenderer;
use bytemuck::{Zeroable, Pod};

pub mod pipeline;

pub struct EntityInstancesSSBO {
    pub instances: Vec<EntityRenderInstance>
}

impl EntityInstancesSSBO {

    pub fn upload(&self, wm: &WmRenderer) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = wm.wgpu_state.device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.instances[..]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE
            }
        );
        
        let pipelines = wm.pipelines.load();

        let bind_group = wm.wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipelines.layouts.instanced_entity_storage,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ]
        });

        (buffer, bind_group)
    }

}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EntityVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub part_id: u32
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
                    format: wgpu::VertexFormat::Uint32
                }
            ],
        }
    }
}

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
/// Data to describe an instance of an entity type on the GPU
pub struct EntityRenderInstance {
    /// Index into the mat4[][]
    pub entity_index: u32,
    /// Index into the float2[] to describe the offset of this entities texture
    pub entity_texture_index: u32,
    /// Used to describe where this entity is standing and where its looking
    pub rotation_and_looking: [[f32; 4]; 4]
}

impl EntityRenderInstance {

    const VAA: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        4 => Uint32,
        5 => Uint32,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
        9 => Float32x4
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {

        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<EntityVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VAA
        }
    }

}
