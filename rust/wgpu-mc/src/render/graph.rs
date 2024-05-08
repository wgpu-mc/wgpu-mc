use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use wgpu::{
    BindGroup, Color, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerBindingType, StoreOp,
};
use wgpu::util::DrawIndirectArgs;

use crate::mc::chunk::RenderLayer;
use crate::mc::entity::InstanceVertex;
use crate::mc::resource::ResourcePath;
use crate::mc::Scene;
use crate::render::entity::EntityVertex;
use crate::render::pipeline::{QuadVertex, BLOCK_ATLAS};
use crate::render::shader::WgslShader;
use crate::render::shaderpack::{
    BindGroupDef, LonghandResourceConfig, PipelineConfig, ShaderPackConfig,
    ShorthandResourceConfig, TypeResourceConfig,
};
use crate::render::sky::{SkyVertex, SunMoonVertex};
use crate::texture::TextureAndView;
use crate::util::WmArena;
use crate::WmRenderer;

pub trait Geometry {
    fn render<'graph: 'pass + 'arena, 'pass, 'arena: 'pass>(
        &mut self,
        wm: &WmRenderer,
        render_graph: &'graph RenderGraph,
        bound_pipeline: &'graph BoundPipeline,
        render_pass: &mut wgpu::RenderPass<'pass>,
        arena: &WmArena<'arena>,
    );
}

pub enum ResourceBacking {
    Buffer(Arc<wgpu::Buffer>, wgpu::BufferBindingType),
    BufferArray(Vec<Arc<wgpu::Buffer>>),
    Texture2D(Arc<TextureAndView>),
    Sampler(Arc<wgpu::Sampler>),
}

impl ResourceBacking {
    pub fn get_bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        match self {
            ResourceBacking::Buffer(_, buffer_ty) => wgpu::BindGroupLayoutEntry {
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
            ResourceBacking::Buffer(buffer, buffer_ty) => vec![wgpu::BindGroupEntry {
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

pub struct BoundPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<(u32, WmBindGroup)>,
    pub config: PipelineConfig,
}

pub struct RenderGraph {
    pub config: ShaderPackConfig,
    pub pipelines: HashMap<String, BoundPipeline>,
    pub resources: HashMap<String, ResourceBacking>,
}

impl RenderGraph {
    fn create_pipelines(
        &mut self,
        wm: &WmRenderer,
        custom_bind_groups: Option<HashMap<String, &wgpu::BindGroupLayout>>,
        geometry_vertex_layouts: Option<HashMap<String, Vec<wgpu::VertexBufferLayout>>>,
    ) {
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
                    BindGroupDef::Resource(resource) => {
                        match (&resource[..], &custom_bind_groups) {
                            ("@bg_ssbo_chunks", _) => {
                                wm.bind_group_layouts.get("ssbo").unwrap()
                            }
                            (_, Some(custom)) => {
                                if let Some(entry) = custom.get(resource) {
                                    entry
                                } else {
                                    unimplemented!()
                                }
                            }
                            (_, None) => unimplemented!(),
                        }
                    }
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

            let push_constants = pipeline_config
                .push_constants
                .iter()
                .map(|(index, name)| {
                    let index = *index as u32;

                    match &name[..] {
                        "@pc_mat4_model" => wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: index..index + 64,
                        },
                        "@pc_section_position" => wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: index..index + 96,
                        },
                        "@pc_total_sections" => wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: index..index + 4,
                        },
                        _ => unimplemented!(),
                    }
                })
                .collect::<Vec<wgpu::PushConstantRange>>();

            let layout =
                wm.wgpu_state
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &bind_group_layouts,
                        push_constant_ranges: &push_constants,
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
                _ => {
                    match geometry_vertex_layouts
                        .as_ref()
                        .map(|layouts| layouts.get(&pipeline_config.geometry))
                        .flatten()
                    {
                        None => unimplemented!(),
                        Some(layout) => Some(layout.clone()),
                    }
                }
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
                    config: pipeline_config.clone(),
                },
            );
        }
    }

    pub fn new(
        wm: &WmRenderer,
        config: ShaderPackConfig,
        mut resources: HashMap<String, ResourceBacking>,
        custom_bind_groups: Option<HashMap<String, &wgpu::BindGroupLayout>>,
        custom_geometry: Option<HashMap<String, Vec<wgpu::VertexBufferLayout>>>,
    ) -> Self {
        for (resource_id, shorthand) in &config.resources.resources {
            match shorthand {
                ShorthandResourceConfig::Int(_) => {}
                ShorthandResourceConfig::Float(_) => {}
                ShorthandResourceConfig::Mat3(_) => {}
                ShorthandResourceConfig::Mat4(_) => {}
                ShorthandResourceConfig::Longhand(LonghandResourceConfig { typed, .. }) => {
                    match typed {
                        TypeResourceConfig::Blob { .. } => {}
                        TypeResourceConfig::Texture3d { .. } => {}
                        TypeResourceConfig::Texture2d { src } => {
                            let bytes = wm
                                .mc
                                .resource_provider
                                .get_bytes(&ResourcePath::from(&src[..]))
                                .unwrap();

                            let tav = TextureAndView::from_image_file_bytes(
                                &wm.wgpu_state,
                                &bytes,
                                resource_id,
                            )
                            .unwrap();

                            resources.insert(
                                resource_id.clone(),
                                ResourceBacking::Texture2D(Arc::new(tav)),
                            );
                        }
                        TypeResourceConfig::TextureDepth => {}
                        TypeResourceConfig::F32 { .. } => {}
                        TypeResourceConfig::F64 { .. } => {}
                        TypeResourceConfig::I64 { .. } => {}
                        TypeResourceConfig::I32 { .. } => {}
                        TypeResourceConfig::Mat3(_) => {}
                        TypeResourceConfig::Mat4(_) => {}
                    }
                }
            }
        }

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
                "@texture_block_atlas".into(),
                ResourceBacking::Texture2D(block_atlas.texture.clone()),
            ),
            (
                "@sampler".into(),
                ResourceBacking::Sampler(wm.mc.texture_manager.default_sampler.clone()),
            ),
        ]);

        graph.create_pipelines(wm, custom_bind_groups, custom_geometry);

        graph
    }

    pub fn render(
        &self,
        wm: &WmRenderer,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
        render_target: &wgpu::TextureView,
        clear_color: [u8; 3],
        geometry: &mut HashMap<String, Box<dyn Geometry>>,
    ) {
        let arena = WmArena::new(4096);

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
                    // for section in scene.chunk_sections.iter() {
                    //     let pos = section.key();
                    //
                    //     let buffers = match &section.buffers {
                    //         None => continue,
                    //         Some(buffers) => buffers,
                    //     };

                    let mut indirect: Vec<DrawIndirectArgs> = vec![];

                    let mut sections = scene.chunk_sections.write();

                    for (pos, section_lock) in sections.iter_mut() {
                        let section = section_lock.get_mut();

                        if let Some(layer) = section.layers.get(&RenderLayer::Solid) {
                            indirect.push(
                                DrawIndirectArgs {
                                    vertex_count: layer.index_range.end - layer.index_range.start,
                                    instance_count: 1,
                                    first_vertex: 0,
                                    first_instance: layer.index_range.start,
                                }
                            );
                        }
                    }

                    let indirect_bytes: Vec<u8> = indirect.iter().map(|args| args.as_bytes()).flatten().copied().collect();

                    wm.wgpu_state.queue.write_buffer(&scene.indirect_buffer, 0, &indirect_bytes);

                    render_pass.set_pipeline(&bound_pipeline.pipeline);

                    set_push_constants(
                        pipeline_config,
                        &mut render_pass,
                        None,
                    );

                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Resource(name) => match &name[..] {
                                "@bg_ssbo_chunks" => {
                                    // #[cfg(not(feature = "vbo-fallback"))]
                                    render_pass.set_bind_group(*index, &scene.chunk_buffer.bind_group, &[]);

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

                    render_pass.set_vertex_buffer(0, scene.chunk_buffer.buffer.slice(..));

                    render_pass.multi_draw_indirect(
                        &scene.indirect_buffer,
                        0,
                        indirect.len() as u32
                    );
                }
                _ => match geometry.get_mut(&pipeline_config.geometry) {
                    None => unimplemented!("Unknown geometry {}", &pipeline_config.geometry),
                    Some(geometry) => {
                        geometry.render(wm, self, bound_pipeline, &mut render_pass, &arena);
                    }
                },
            }
        }
    }
}

pub fn set_push_constants(
    pipeline: &PipelineConfig,
    render_pass: &mut wgpu::RenderPass,
    push_constants: Option<HashMap<String, (Vec<u8>, wgpu::ShaderStages)>>,
) {
    pipeline
        .push_constants
        .iter()
        .for_each(|(offset, resource)| {
            match push_constants
                .as_ref()
                .map(|others| others.get(resource))
                .flatten()
            {
                None => unimplemented!("Unknown push constant resource value"),
                Some((data, stages)) => {
                    render_pass.set_push_constants(*stages, *offset as u32, data)
                }
            }
        });
}
