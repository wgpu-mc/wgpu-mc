use wgpu::{RenderPipelineDescriptor, BindGroupLayout};
use crate::render::shader::Shader;

pub struct Shaders {
    pub sky: Shader,
    pub terrain: Shader,
    pub grass: Shader,
    pub transparent: Shader,
}

pub struct Pipelines {
    pub sky_pipeline: wgpu::RenderPipeline,
    pub terrain_pipeline: wgpu::RenderPipeline,
    pub grass_pipeline: wgpu::RenderPipeline,
    pub transparent_pipeline: wgpu::RenderPipeline,
    
    pub layouts: Layouts,
    pub shaders: Shaders
}

pub struct Layouts {
    pub texture_bind_group_layout: BindGroupLayout,
    pub cubemap_bind_group_layout: BindGroupLayout,
    pub camera_bind_group_layout: BindGroupLayout
}

impl Pipelines {
    
    fn create_bind_group_layouts(device: &wgpu::Device) -> Layouts {
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            }
        );

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout Descriptor"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false
                        },
                        count: None
                    }
                ]
            }
        );

        let cubemap_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Cubemap Bind Group Layout Descriptor"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false
                        },
                        count: None
                    }
                ]
            }
        );

        Layouts {
            texture_bind_group_layout,
            cubemap_bind_group_layout,
            camera_bind_group_layout
        }
    }

    fn create_pipeline_layouts(device: &wgpu::Device, layouts: &Layouts) -> (wgpu::PipelineLayout, wgpu::PipelineLayout, wgpu::PipelineLayout, wgpu::PipelineLayout) {
        (
            device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Sky Pipeline Layout"),
                    bind_group_layouts: &[
                        &layouts.cubemap_bind_group_layout, &layouts.camera_bind_group_layout
                    ],
                    push_constant_ranges: &[]
                }
            ),
            device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Terrain Pipeline Layout"),
                    bind_group_layouts: &[
                        &layouts.texture_bind_group_layout, &layouts.cubemap_bind_group_layout, &layouts.camera_bind_group_layout
                    ],
                    push_constant_ranges: &[]
                }
            ),device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Grass Pipeline Layout"),
                bind_group_layouts: &[
                    &layouts.texture_bind_group_layout, &layouts.cubemap_bind_group_layout, &layouts.camera_bind_group_layout
                ],
                push_constant_ranges: &[]
            }
        ),device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Transparent Pipeline Layout"),
                bind_group_layouts: &[
                    &layouts.texture_bind_group_layout, &layouts.cubemap_bind_group_layout, &layouts.camera_bind_group_layout
                ],
                push_constant_ranges: &[]
            }
        ),
        )
    }

    pub fn init(device: &wgpu::Device, shaders: Shaders) -> Self {
        let bg_layouts = Self::create_bind_group_layouts(device);
        let pipeline_layouts = Self::create_pipeline_layouts(device, &bg_layouts);
        
        Self {
            sky_pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layouts.0),
                vertex: wgpu::VertexState {
                    module: &shaders.sky.vert,
                    entry_point: "main",
                    buffers: &[]
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    clamp_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false
                },
                //TODO: probably don't need a depth stencil (this is a reminder in case I do)
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders.sky.frag,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE
                        }),
                        write_mask: Default::default()
                    }]
                })
            }),
            terrain_pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layouts.1),
                vertex: wgpu::VertexState {
                    module: &shaders.terrain.vert,
                    entry_point: "main",
                    buffers: &[]
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    clamp_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default()
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders.terrain.frag,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE
                        }),
                        write_mask: Default::default()
                    }]
                })
            }),
            grass_pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layouts.2),
                vertex: wgpu::VertexState {
                    module: &shaders.grass.vert,
                    entry_point: "main",
                    buffers: &[]
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    clamp_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default()
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders.grass.frag,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE
                        }),
                        write_mask: Default::default()
                    }]
                })
            }),
            transparent_pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layouts.0),
                vertex: wgpu::VertexState {
                    module: &shaders.transparent.vert,
                    entry_point: "main",
                    buffers: &[]
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    clamp_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default()
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders.transparent.frag,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE
                        }),
                        write_mask: Default::default()
                    }]
                })
            }),
            layouts: bg_layouts,
            shaders
        }

    }

}
