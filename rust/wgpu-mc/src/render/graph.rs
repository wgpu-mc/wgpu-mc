use glam::{IVec2, IVec3};
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;
use treeculler::Vec3;

use crate::mc::chunk::{ChunkBuffers, RenderLayer, Section, SECTIONS_PER_CHUNK};
use wgpu::{
    Color, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerBindingType, StoreOp,
};

use crate::mc::entity::InstanceVertex;
use crate::mc::resource::ResourcePath;
use crate::mc::{MinecraftState, Scene};
use crate::render::entity::EntityVertex;
use crate::render::pipeline::{QuadVertex, BLOCK_ATLAS};
use crate::render::shader::WgslShader;
use crate::render::shaderpack::{BindGroupDef, PipelineConfig, ShaderPackConfig};
use crate::render::sky::{SkyVertex, SunMoonVertex};
use crate::texture::TextureAndView;
use crate::util::WmArena;
use crate::WmRenderer;

pub enum ResourceBacking {
    BufferBacked(Arc<wgpu::Buffer>, wgpu::BufferBindingType),
    BufferArray(Vec<Arc<wgpu::Buffer>>),
    Texture2D(Arc<TextureAndView>),
    Sampler(Arc<wgpu::Sampler>),
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
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            ResourceBacking::Texture2D(_) => wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            ResourceBacking::Sampler(_) => wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
        }
    }

    pub fn get_bind_group_entries(&self, index: u32) -> Vec<wgpu::BindGroupEntry> {
        match self {
            ResourceBacking::BufferBacked(buffer, buffer_ty) => vec![wgpu::BindGroupEntry {
                binding: index,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
            ResourceBacking::Texture2D(texture) => vec![wgpu::BindGroupEntry {
                binding: index,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            }],
            ResourceBacking::Sampler(sampler) => vec![wgpu::BindGroupEntry {
                binding: index,
                resource: wgpu::BindingResource::Sampler(sampler),
            }],
            // RenderResource::TextureHandle(handle) => vec![
            //     wgpu::BindGroupEntry {
            //         binding: index,
            //         resource: wgpu::BindingResource::TextureView(handle.),
            //     }
            // ],
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum WmBindGroup {
    Resource(String),
    Custom(wgpu::BindGroup),
}

struct BoundPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<(u32, WmBindGroup)>,
}

pub struct RenderGraph {
    pub config: ShaderPackConfig,
    pub pipelines: HashMap<String, BoundPipeline>,
    pub resources: HashMap<String, ResourceBacking>,
}

impl RenderGraph {
    fn create_pipelines(&mut self, wm: &WmRenderer) {
        self.pipelines.clear();

        let arena = WmArena::new(1024);

        for (pipeline_name, pipeline_config) in &self.config.pipelines.pipelines {
            let bind_group_layouts = pipeline_config
                .bind_groups
                .iter()
                .map(|(slot, def)| match def {
                    BindGroupDef::Entries(entries) => {
                        let layout_entries = entries
                            .iter()
                            .map(|(index, resource_id)| {
                                let resource = self.resources.get(resource_id).unwrap();
                                resource.get_bind_group_layout_entry(*index as u32)
                            })
                            .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

                        &*arena.alloc(wm.wgpu_state.device.create_bind_group_layout(
                            &wgpu::BindGroupLayoutDescriptor {
                                label: None,
                                entries: &layout_entries,
                            },
                        ))
                    }
                    BindGroupDef::Resource(resource) => match &resource[..] {
                        "@bg_chunk_ssbos" => wm.bind_group_layouts.get("chunk_ssbos").unwrap(),
                        _ => unimplemented!(),
                    },
                })
                .collect::<Vec<&wgpu::BindGroupLayout>>();

            let wm_bind_groups = pipeline_config
                .bind_groups
                .iter()
                .enumerate()
                .map(|(vec_index, (slot, def))| match def {
                    BindGroupDef::Entries(entries) => {
                        let entries = entries
                            .iter()
                            .map(|(index, resource_id)| {
                                let resource = self.resources.get(resource_id).unwrap();
                                resource.get_bind_group_entries(*index as u32)
                            })
                            .flatten()
                            .collect::<Vec<wgpu::BindGroupEntry>>();

                        let bind_group =
                            wm.wgpu_state
                                .device
                                .create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: None,
                                    layout: &bind_group_layouts[vec_index],
                                    entries: &entries,
                                });

                        (*slot as u32, WmBindGroup::Custom(bind_group))
                    }
                    BindGroupDef::Resource(resource) => {
                        (*slot as u32, WmBindGroup::Resource(resource.clone()))
                    }
                })
                .collect::<Vec<(u32, WmBindGroup)>>();

            let layout =
                wm.wgpu_state
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &bind_group_layouts,
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
                #[cfg(not(feature = "vbo-fallback"))]
                "@geo_terrain" => None,
                #[cfg(feature = "vbo-fallback")]
                "@geo_terrain" => {
                    const VAA: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
                        0 => Uint32,
                        1 => Uint32,
                        2 => Uint32,
                        3 => Uint32
                    ];

                    Some(vec![wgpu::VertexBufferLayout {
                        array_stride: (mem::size_of::<u32>() * 4) as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &VAA,
                    }])
                }
                "@geo_entities" => Some(vec![EntityVertex::desc(), InstanceVertex::desc()]),
                "@geo_quad" => Some(vec![QuadVertex::desc()]),
                "@geo_sun_moon" => Some(vec![SunMoonVertex::desc()]),
                "@geo_sky_scatter" | "@geo_sky_stars" | "@geo_sky_fog" => {
                    Some(vec![SkyVertex::desc()])
                }
                _ => unimplemented!(),
            };

            let label = format!("{}", pipeline_name);

            let render_pipeline =
                wm.wgpu_state
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&label),
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: &shader.module,
                            entry_point: "vert",
                            compilation_options: Default::default(),
                            buffers: match &vertex_buffer {
                                None => &[],
                                Some(buffer_layout) => buffer_layout,
                            },
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Cw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: Default::default(),
                            conservative: false,
                        },
                        depth_stencil: pipeline_config.depth.as_ref().map(|_| {
                            wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: true,
                                depth_compare: wgpu::CompareFunction::Less,
                                stencil: wgpu::StencilState::default(),
                                bias: Default::default(),
                            }
                        }),
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: &shader.module,
                            entry_point: "frag",
                            compilation_options: Default::default(),
                            targets: &pipeline_config
                                .output
                                .iter()
                                .map(|_| {
                                    Some(wgpu::ColorTargetState {
                                        format: wgpu::TextureFormat::Bgra8Unorm,
                                        blend: Some(match &pipeline_config.blending[..] {
                                            "alpha_blending" => wgpu::BlendState::ALPHA_BLENDING,
                                            "premultiplied_alpha_blending" => {
                                                wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING
                                            }
                                            "color_add_alpha_blending" => wgpu::BlendState {
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
                                            },
                                            _ => unimplemented!("Unknown blend state"),
                                        }),
                                        write_mask: Default::default(),
                                    })
                                })
                                .collect::<Vec<_>>(),
                        }),
                        multiview: None,
                    });

            self.pipelines.insert(
                pipeline_name.clone(),
                BoundPipeline {
                    pipeline: render_pipeline,
                    bind_groups: wm_bind_groups,
                },
            );
        }
    }

    pub fn new(
        wm: &WmRenderer,
        resources: HashMap<String, ResourceBacking>,
        config: ShaderPackConfig,
    ) -> Self {
        let mut graph = Self {
            config,
            pipelines: HashMap::new(),
            resources,
        };

        let atlases = wm.mc.texture_manager.atlases.load();

        let atlas_swap = atlases.get(BLOCK_ATLAS).unwrap();
        let block_atlas = atlas_swap.load();

        graph.resources.extend([
            (
                "@tex_block_atlas".into(),
                ResourceBacking::Texture2D(block_atlas.texture.clone()),
            ),
            (
                "@sampler".into(),
                ResourceBacking::Sampler(wm.mc.texture_manager.default_sampler.clone()),
            ),
        ]);

        graph.create_pipelines(wm);

        graph
    }

    pub fn render(
        &self,
        wm: &WmRenderer,
        scene: &Scene,
        render_target: &wgpu::TextureView,
        clear_color: [u8; 3],
    ) {
        let arena = WmArena::new(4096);

        let mut encoder = wm
            .wgpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut should_clear_depth = true;

        for (pipeline_name, bound_pipeline) in &self.pipelines {
            let pipeline_config = self.config.pipelines.pipelines.get(pipeline_name).unwrap();

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
                                "@framebuffer_texture" => render_target,
                                _ => unimplemented!(),
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

                    let depth_view =
                        if depth_texture == "@texture_depth" {
                            arena.alloc(scene.depth_texture.create_view(
                                &wgpu::TextureViewDescriptor {
                                    label: None,
                                    format: Some(wgpu::TextureFormat::Depth32Float),
                                    dimension: Some(wgpu::TextureViewDimension::D2),
                                    aspect: Default::default(),
                                    base_mip_level: 0,
                                    mip_level_count: None,
                                    base_array_layer: 0,
                                    array_layer_count: None,
                                },
                            ))
                        } else {
                            match self.resources.get(depth_texture) {
                                Some(&ResourceBacking::Texture2D(ref view)) => &view.view,
                                _ => unimplemented!("Unknown depth target {}", depth_texture),
                            }
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
                "@geo_terrain" => {
                    for section in scene.chunk_sections.iter() {
                        let section = arena.alloc(section);

                        let pos = section.key();

                        let buffers = match &section.buffers {
                            None => continue,
                            Some(buffers) => buffers,
                        };

                        render_pass.set_pipeline(&bound_pipeline.pipeline);

                        for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                            match bind_group {
                                WmBindGroup::Resource(name) => match &name[..] {
                                    "@bg_chunk_ssbos" => {
                                        #[cfg(not(feature = "vbo-fallback"))]
                                        if let Some(ref buffers) = section.buffers {
                                            render_pass.set_bind_group(
                                                *index,
                                                &buffers.bind_group,
                                                &[],
                                            );
                                        }

                                        #[cfg(feature = "vbo-fallback")]
                                        {
                                            panic!("SSBOs are not supported on WebGL")
                                        }
                                    }
                                    _ => unimplemented!(),
                                },
                                WmBindGroup::Custom(bind_group) => {
                                    render_pass.set_bind_group(*index, bind_group, &[]);
                                }
                            }
                        }

                        if let Some(solid) = section.layers.get(&RenderLayer::Solid) {
                            #[cfg(not(feature = "vbo-fallback"))]
                            {
                                render_pass.draw(solid.clone(), 0..1);
                            }

                            #[cfg(feature = "vbo-fallback")]
                            if let Some(ref buffers) = section.buffers {
                                render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
                                render_pass.set_index_buffer(
                                    buffers.index_buffer.slice(..),
                                    IndexFormat::Uint32,
                                );

                                render_pass.draw_indexed(solid.clone(), 0, 0..1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        wm.wgpu_state.queue.submit([encoder.finish()]);
    }
}

pub fn set_push_constants(
    scene: &Scene,
    mc_state: &MinecraftState,
    pipeline: &PipelineConfig,
    render_pass: &mut wgpu::RenderPass,
    section: Option<&Section>,
    surface_config: &wgpu::SurfaceConfiguration,
    chunk_offset: IVec2,
    section_pos: Option<IVec3>,
    parts_per_entity: Option<u32>,
) {
    let sky = &scene.sky_state;
    let render_effects = &scene.render_effects;

    //janky way of "still loading boi!"
    if render_effects.fog_color.is_empty() {
        return;
    }
    pipeline
        .push_constants
        .iter()
        .for_each(|(offset, resource)| match &resource[..] {
            "@pc_framebuffer_size" => {
                render_pass.set_push_constants(
                    wgpu::ShaderStages::FRAGMENT,
                    *offset as u32,
                    bytemuck::cast_slice(&[
                        surface_config.width as f32,
                        surface_config.height as f32,
                    ]),
                );
            }
            "@pc_chunk_position" => render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                *offset as u32,
                bytemuck::cast_slice(&[
                    section_pos.unwrap().x + chunk_offset.x,
                    section_pos.unwrap().y,
                    section_pos.unwrap().z + chunk_offset.y,
                ]),
            ),
            "@pc_parts_per_entity" => render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                *offset as u32,
                bytemuck::cast_slice(&[parts_per_entity.unwrap()]),
            ),
            "@pc_environment_data" => render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                *offset as u32,
                // bytemuck::cast_slice(&[
                //     sky.angle,
                //     sky.brightness,
                //     sky.star_shimmer,
                //     render_effects.fog_start,
                //     render_effects.fog_end,
                //     render_effects.fog_shape,
                //     render_effects.fog_color[0],
                //     render_effects.fog_color[1],
                //     render_effects.fog_color[2],
                //     render_effects.fog_color[3],
                //     sky.color[0],
                //     sky.color[1],
                //     sky.color[2],
                //     render_effects.dimension_fog_color[0],
                //     render_effects.dimension_fog_color[1],
                //     render_effects.dimension_fog_color[2],
                //     render_effects.dimension_fog_color[3],
                // ]),
                &[],
            ),
            _ => unimplemented!("Unknown push constant resource value"),
        });
}
