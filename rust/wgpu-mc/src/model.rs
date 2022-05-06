use crate::texture::TextureSamplerView;
use crate::{texture, WgpuState};

use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

use crate::render::pipeline::RenderPipelineManager;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GuiVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl GuiVertex {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GuiVertex>() as wgpu::BufferAddress,
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
            ],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

///Represents a texture that has been uploaded to GPU and has an associated `BindGroup`
#[derive(Debug)]
pub struct BindableTexture {
    pub tsv: texture::TextureSamplerView,
    pub bind_group: wgpu::BindGroup,
}

impl BindableTexture {
    #[must_use]
    pub fn from_tsv(
        wgpu_state: &WgpuState,
        pipelines: &RenderPipelineManager,

        texture: TextureSamplerView,
    ) -> Self {
        let bind_group = wgpu_state.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: pipelines.bind_group_layouts.read().get("texture").unwrap(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        Self {
            tsv: texture,
            bind_group,
        }
    }
}
