use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::access::Access;
use arc_swap::ArcSwapAny;
use wgpu::{Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp};
use crate::mc::resource::ResourcePath;
use crate::render::shader::WgslShader;
use crate::render::shaderpack::{BindGroupDef, LonghandResourceConfig, ShaderPackConfig, ShorthandResourceConfig, TypeResourceConfig};
use crate::{WgpuState, WmRenderer};
use crate::mc::entity::InstanceVertex;
use crate::mc::World;
use crate::render::entity::EntityVertex;
use crate::render::pipeline::QuadVertex;
use crate::render::sky::{SkyVertex, SunMoonVertex};
use crate::texture::{BindableTexture, TextureAndView, TextureHandle};
use crate::util::WmArena;

enum ResourceBacking {
    BufferBacked(Arc<wgpu::Buffer>, wgpu::BufferBindingType),
    BufferArray(Vec<Arc<wgpu::Buffer>>),
    Texture2D(Arc<TextureAndView>),
    TextureHandle(TextureHandle),
    Sampler(Arc<wgpu::Sampler>)
}

impl ResourceBacking {

    pub fn get_bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        match self {
            ResourceBacking::BufferBacked(_, buffer_ty) => wgpu::BindGroupLayoutEntry {
                binding,
                //TODO
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: *buffer_ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            ResourceBacking::BufferArray(buffers) => wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: true,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            ResourceBacking::Texture2D(_) => wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            ResourceBacking::TextureHandle(handle) => wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            _ => unimplemented!()
        }
    }

    pub fn get_bind_group_entries(&self, index: u32) -> Vec<wgpu::BindGroupEntry> {
        match self {
            ResourceBacking::BufferBacked(buffer, buffer_ty) => vec![
                wgpu::BindGroupEntry {
                    binding: index,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                }
            ],
            ResourceBacking::Texture2D(texture) => vec![
                wgpu::BindGroupEntry {
                    binding: index,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                }
            ],
            ResourceBacking::Sampler(sampler) => vec![
                wgpu::BindGroupEntry {
                    binding: index,
                    resource: wgpu::BindingResource::Sampler(sampler),
                }
            ],
            // RenderResource::TextureHandle(handle) => vec![
            //     wgpu::BindGroupEntry {
            //         binding: index,
            //         resource: wgpu::BindingResource::TextureView(handle.),
            //     }
            // ],
            _ => todo!()
        }
    }

}

pub enum WmBindGroup {
    Resource(String),
    Custom(wgpu::BindGroup)
}

struct BoundPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<WmBindGroup>
}

struct RenderGraph {
    pub config: ShaderPackConfig,
    pub pipelines: Vec<BoundPipeline>,
    pub resources: HashMap<String, ResourceBacking>
}

impl RenderGraph {

    fn create_pipelines(&mut self, wm: &WmRenderer) {
        self.pipelines.clear();

        for (pipeline_name, pipeline_config) in &self.config.pipelines.pipelines {
            let bind_group_layouts = pipeline_config.bind_groups.iter().map(|(slot, def)| {

                match def {
                    BindGroupDef::Entries(entries) => {
                        let layout_entries = entries.iter().map(|(index, resource_id)| {
                            let resource = self.resources.get(resource_id).unwrap();
                            resource.get_bind_group_layout_entry(*index as u32)
                        }).collect::<Vec<wgpu::BindGroupLayoutEntry>>();

                        wm.wgpu_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &layout_entries,
                        })
                    }
                    BindGroupDef::Resource(resource) => match &resource[..] {
                        _ => todo!()
                    }
                }
            }).collect::<Vec<wgpu::BindGroupLayout>>();

            let wm_bind_groups = pipeline_config.bind_groups.iter().enumerate().map(|(vec_index, (slot, def))| {
                match def {
                    BindGroupDef::Entries(entries) => {
                        let entries = entries.iter().map(|(index, resource_id)| {
                            let resource = self.resources.get(resource_id).unwrap();
                            resource.get_bind_group_entries(*index as u32)
                        }).flatten().collect::<Vec<wgpu::BindGroupEntry>>();

                        let bind_group = wm.wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &bind_group_layouts[vec_index],
                            entries: &entries,
                        });

                        WmBindGroup::Custom(bind_group)
                    }
                    BindGroupDef::Resource(resource) => WmBindGroup::Resource(resource.clone())
                }
            }).collect::<Vec<WmBindGroup>>();

            let borrowed_layouts = bind_group_layouts.iter().collect::<Vec<&wgpu::BindGroupLayout>>();

            let layout = wm.wgpu_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &borrowed_layouts,
                push_constant_ranges: &[],
            });

            let shader = WgslShader::init(
                &ResourcePath(format!("wgpu_mc:shaders/{}.wgsl", pipeline_name)),
                &*wm.mc.resource_provider,
                &wm.wgpu_state.device,
                "frag".into(),
                "vert".into(),
            )
                .unwrap();

            let vertex_buffer = match &pipeline_config.geometry[..] {
                "wm_geo_terrain" => None,
                "wm_geo_entities" => {
                    Some(vec![EntityVertex::desc(), InstanceVertex::desc()])
                }
                "wm_geo_quad" => Some(vec![QuadVertex::desc()]),
                "wm_geo_sun_moon" => Some(vec![SunMoonVertex::desc()]),
                "wm_geo_sky_scatter" | "wm_geo_sky_stars" | "wm_geo_sky_fog" => Some(vec![SkyVertex::desc()]),
                _ => unimplemented!()
            };

            let label = format!("{}", pipeline_name);

            let render_pipeline = wm.wgpu_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: "vert",
                    buffers: match &vertex_buffer {
                        None => &[],
                        Some(buffer_layout) => buffer_layout
                    },
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: "frag",
                    targets: &pipeline_config
                        .output
                        .iter()
                        .map(|_| {
                            Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(match &pipeline_config.blending[..] {
                                    "alpha_blending" => {
                                        wgpu::BlendState::ALPHA_BLENDING
                                    }
                                    "premultiplied_alpha_blending" => {
                                        wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING
                                    }
                                    "color_add_alpha_blending" => {
                                        wgpu::BlendState {
                                            color: wgpu::BlendComponent {
                                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                                dst_factor: wgpu::BlendFactor::One,
                                                operation: wgpu::BlendOperation::Add,
                                            },
                                            alpha: wgpu::BlendComponent {
                                                src_factor: wgpu::BlendFactor::One,
                                                dst_factor: wgpu::BlendFactor::Zero,
                                                operation: wgpu::BlendOperation::Add,
                                            },
                                        }
                                    }
                                    _ => unimplemented!("Unknown blend state"),
                                }),
                                write_mask: Default::default(),
                            })
                        })
                        .collect::<Vec<_>>(),
                }),
                multiview: None,
            });

            self.pipelines.push(BoundPipeline {
                pipeline: render_pipeline,
                bind_groups: wm_bind_groups,
            });
        }

    }

    fn new(wm: &WmRenderer, config: ShaderPackConfig) -> Self {
        let mut graph = Self {
            config,
            pipelines: vec![],
            resources: Default::default(),
        };

        graph.create_pipelines(wm);

        graph
    }

    fn render(&self, wm: &WmRenderer, world: &World, render_target: &wgpu::TextureView, clear_color: [u8; 3]) {

        let mut arena = WmArena::new(4096);

        let mut encoder = wm.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        let mut should_clear_depth = false;

        for (pipeline_name, pipeline_config) in &self.config.pipelines.pipelines {

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &pipeline_config
                    .output
                    .iter()
                    .map(|texture_name| {
                        let resource_definition = self.config.resources.resources.get(texture_name);

                        Some(RenderPassColorAttachment {
                            view: match &texture_name[..] {
                                "wm_framebuffer_texture" => render_target,
                                _ => unimplemented!()
                            },
                            resolve_target: None,
                            ops: Operations {
                                load: if !pipeline_config.clear {
                                    LoadOp::Load
                                } else {
                                    LoadOp::Clear(Color {
                                        r: clear_color[0] as f64,
                                        g: clear_color[1] as f64,
                                        b: clear_color[2] as f64,
                                        a: 1.0,
                                    })
                                },
                                store: StoreOp::Store,
                            },
                        })
                    })
                    .collect::<Vec<_>>(),
                depth_stencil_attachment: pipeline_config.depth.as_ref().map(|depth_texture| {
                    let will_clear_depth = should_clear_depth;
                    should_clear_depth = false;

                    let depth_view = match self.resources.get(depth_texture) {
                        Some(&ResourceBacking::TextureHandle(ref handle)) => {
                            let bindable_texture = arena.alloc(handle.clone().bindable_texture.load_full());
                            &bindable_texture.tv.view
                        },
                        Some(&ResourceBacking::Texture2D(ref view)) => {
                            &view.view
                        }
                        _ => unimplemented!("Unknown depth target {}", depth_texture),
                    };

                    RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(Operations {
                            load: if will_clear_depth {
                                LoadOp::Clear(1.0)
                            } else {
                                LoadOp::Load
                            },
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
            });

            match &pipeline_config.geometry[..] {
                "wm_geo_terrain" => {



                },
                _ => {

                }
            }

        }

    }

}