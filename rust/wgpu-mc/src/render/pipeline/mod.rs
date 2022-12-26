pub mod debug_lines;

use crate::render::shader::WmShader;
use wgpu::{BindGroupLayout, ComputePipeline, PipelineLayout, SamplerBindingType};

use crate::mc::chunk::RenderLayer;
use arc_swap::ArcSwap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::WmRenderer;

use crate::mc::resource::ResourceProvider;

use crate::util::WmArena;

use crate::wgpu::RenderPipeline;

pub const BLOCK_ATLAS: &str = "wgpu_mc:atlases/block";
pub const ENTITY_ATLAS: &str = "wgpu_mc:atlases/entity";

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub lightmap_coords: [f32; 2],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub tangent: [f32; 4],
    pub uv_offset: u32,
}

impl Vertex {
    const VAA: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Uint32
    ];

    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VAA,
        }
    }
}

pub struct WmPipelines {
    pub pipeline_layouts: ArcSwap<HashMap<String, Arc<PipelineLayout>>>,
    pub render_pipelines: ArcSwap<HashMap<String, Arc<RenderPipeline>>>,
    pub compute_pipelines: ArcSwap<HashMap<String, Arc<ComputePipeline>>>,

    pub chunk_layers: ArcSwap<Vec<Box<dyn RenderLayer>>>,

    pub shader_map: RwLock<HashMap<String, Box<dyn WmShader>>>,
    pub bind_group_layouts: RwLock<HashMap<String, BindGroupLayout>>,
    pub resource_provider: Arc<dyn ResourceProvider>,
}

impl WmPipelines {
    fn create_bind_group_layouts(device: &wgpu::Device) -> HashMap<String, BindGroupLayout> {
        [
            (
                "camera".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            ),
            (
                "texture_depth".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Depth Texture Descriptor"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                }),
            ),
            (
                "texture".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout Descriptor"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                }),
            ),
            (
                "cubemap".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Cubemap Bind Group Layout Descriptor"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                }),
            ),
            (
                "ssbo".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            ),
            (
                "ssbo_mut".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            ),
            (
                "matrix".into(),
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Matrix Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            ),
        ]
        .into_iter()
        .collect()
    }

    pub fn new(resource_provider: Arc<dyn ResourceProvider>) -> Self {
        Self {
            pipeline_layouts: ArcSwap::new(Arc::new(HashMap::new())),
            render_pipelines: ArcSwap::new(Arc::new(HashMap::new())),
            resource_provider,
            bind_group_layouts: RwLock::new(HashMap::new()),
            shader_map: RwLock::new(HashMap::new()),
            compute_pipelines: ArcSwap::new(Arc::new(HashMap::new())),
            chunk_layers: ArcSwap::new(Arc::new(vec![])),
        }
    }

    pub fn init(&self, wm: &WmRenderer) {
        {
            self.bind_group_layouts
                .write()
                .extend(Self::create_bind_group_layouts(&wm.wgpu_state.device).into_iter())
        }
    }
}
