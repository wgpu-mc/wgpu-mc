use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use cgmath::Rad;
use wgpu::util::BufferInitDescriptor;
use wgpu::{BindGroupDescriptor, BindGroupEntry};

use crate::camera::UniformMatrixHelper;
use crate::render::shader::{WgslShader, WmShader};
use crate::util::WmArena;
use crate::wgpu::util::DeviceExt;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
/// Data to describe an instance of an entity type on the GPU
pub struct DebugLineVertex {
    /// Index into mat4[][]
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl DebugLineVertex {
    const VAA: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DebugLineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VAA,
        }
    }
}

pub const LINES: &[f32] = &[
    //+Y is up in mc
    0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.2, 0.0, 0.0, 1.0, 0.0, //-X is west in mc
    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -0.2, 0.0, 0.0, 1.0, 0.0, 0.0, //-Z is north in mc
    0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -0.2, 0.0, 0.0, 1.0,
];
