use glam::ivec3;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::sync::Arc;
use treeculler::{BVol, Frustum, Vec3, AABB};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use wgpu::{
    BufferUsages, Color, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerBindingType, ShaderStages,
    StoreOp,
};

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

pub trait Geometry: Send + Sync {
    fn render<'graph: 'pass + 'arena, 'pass, 'arena: 'pass>(
        &mut self,
        wm: &WmRenderer,
        render_graph: &'graph RenderGraph,
        bound_pipeline: &'graph BoundPipeline,
        render_pass: &mut wgpu::RenderPass<'pass>,
        arena: &WmArena<'arena>,
    );
}

#[derive(Debug)]
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
                visibility: ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: *buffer_ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            ResourceBacking::BufferArray(_buffers) => wgpu::BindGroupLayoutEntry {
                binding,
                visibility: ShaderStages::all(),
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
            ResourceBacking::Buffer(buffer, _buffer_ty) => vec![wgpu::BindGroupEntry {
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

#[derive(Debug)]
pub struct BoundPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<(u32, WmBindGroup)>,
    pub config: PipelineConfig,
}

#[derive(Debug)]
pub struct RenderGraph {
    pub config: ShaderPackConfig,
    pub pipelines: LinkedHashMap<String, BoundPipeline>,
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
                .map(|(_slot, def)| match def {
                    BindGroupDef::Entries(entries) => {
                        let layout_entries = entries
                            .iter()
                            .map(|(index, resource_id)| {
                                let resource = self.resources.get(resource_id).unwrap();
                                resource.get_bind_group_layout_entry(*index as u32)
                            })
                            .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

                        &*arena.alloc(wm.gpu.device.create_bind_group_layout(
                            &wgpu::BindGroupLayoutDescriptor {
                                label: None,
                                entries: &layout_entries,
                            },
                        ))
                    }
                    BindGroupDef::Resource(resource) => {
                        match (&resource[..], &custom_bind_groups) {
                            ("@bg_ssbo_chunks", _) => wm.bind_group_layouts.get("ssbo").unwrap(),
                            ("@bg_entity", _) => wm.bind_group_layouts.get("entity").unwrap(),
                            (_, Some(custom)) => {
                                if let Some(entry) = custom.get(resource) {
                                    entry
                                } else {
                                    unimplemented!("{}", resource)
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
                            .flat_map(|(index, resource_id)| {
                                let resource = self.resources.get(resource_id).unwrap();
                                resource.get_bind_group_entries(*index as u32)
                            })
                            .collect::<Vec<wgpu::BindGroupEntry>>();

                        let bind_group =
                            wm.gpu
                                .device
                                .create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: None,
                                    layout: bind_group_layouts[vec_index],
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
                            range: index..index + 12,
                        },
                        "@pc_total_sections" => wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: index..index + 4,
                        },
                        "@pc_parts_per_entity" => wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: index..index + 4,
                        },
                        "@pc_electrum_color" => wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::FRAGMENT,
                            range: index..index + 16,
                        },
                        "@pc_environment_data" => wgpu::PushConstantRange {
                            stages: ShaderStages::VERTEX_FRAGMENT,
                            range: index..index + 68,
                        },
                        _ => unimplemented!(),
                    }
                })
                .collect::<Vec<wgpu::PushConstantRange>>();

            let layout =
                wm.gpu
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &bind_group_layouts,
                        push_constant_ranges: &push_constants,
                    });

            let shader = WgslShader::init(
                &ResourcePath(format!("wgpu_mc:shaders/{}.wgsl", pipeline_name)),
                &*wm.mc.resource_provider,
                &wm.gpu.device,
                "frag".into(),
                "vert".into(),
            )
            .unwrap();

            let vertex_buffer = match &pipeline_config.geometry[..] {
                "@geo_terrain" => None,
                "@geo_entities" => Some(vec![EntityVertex::desc(), InstanceVertex::desc()]),
                "@geo_quad" => Some(vec![QuadVertex::desc()]),
                "@geo_sun_moon" => Some(vec![SunMoonVertex::desc()]),
                "@geo_sky_scatter" | "@geo_sky_stars" | "@geo_sky_fog" => {
                    Some(vec![SkyVertex::desc()])
                }
                _ => {
                    match geometry_vertex_layouts
                        .as_ref()
                        .and_then(|layouts| layouts.get(&pipeline_config.geometry))
                    {
                        None => unimplemented!(),
                        Some(layout) => Some(layout.clone()),
                    }
                }
            };

            let label = pipeline_name.to_string();

            let render_pipeline =
                wm.gpu
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
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
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
                                            "replace" => wgpu::BlendState::REPLACE,
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
                        cache: None,
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
                                &wm.gpu,
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
            pipelines: LinkedHashMap::new(),
            resources,
        };

        let atlases = wm.mc.texture_manager.atlases.read();

        let block_atlas = atlases.get(BLOCK_ATLAS).unwrap();

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
        clear_color: [f32; 3],
        geometry: &mut HashMap<String, Box<dyn Geometry>>,
        frustum: &Frustum<f32>,
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

                    let depth_view = if depth_texture == "@texture_depth" {
                        arena.alloc(scene.depth_texture.read().create_view(
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
                            Some(ResourceBacking::Texture2D(view)) => &view.view,
                            _ => unimplemented!("Unknown depth target {}", depth_texture),
                        }
                    };

                    RenderPassDepthStencilAttachment {
                        view: depth_view,
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
                    render_pass.set_pipeline(&bound_pipeline.pipeline);

                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Resource(name) => match &name[..] {
                                "@bg_ssbo_chunks" => {
                                    render_pass.set_bind_group(
                                        *index,
                                        &scene.chunk_buffer.bind_group,
                                        &[],
                                    );
                                }
                                _ => unimplemented!(),
                            },
                            WmBindGroup::Custom(bind_group) => {
                                render_pass.set_bind_group(*index, bind_group, &[]);
                            }
                        }
                    }

                    render_pass.set_index_buffer(
                        scene.chunk_buffer.buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    let sections = scene.section_storage.write();
                    let camera_pos = *scene.camera_section_pos.read();
                    for (pos, section) in sections.iter() {
                        let rel_pos = ivec3(pos.x - camera_pos.x, pos.y, pos.z - camera_pos.y);
                        let a: Vec3<f32> =
                            [rel_pos.x as f32, rel_pos.y as f32, rel_pos.z as f32].into();
                        let b: Vec3<f32> = a + Vec3::new(1.0, 1.0, 1.0);

                        let bounds: AABB<f32> =
                            AABB::new((a * 16.0).into_array(), (b * 16.0).into_array());

                        if !bounds.coherent_test_against_frustum(frustum, 0).0 {
                            continue;
                        }
                        if let Some(layer) = &section.layers[RenderLayer::Solid as usize] {
                            let mut pc: HashMap<String, (Vec<u8>, ShaderStages)> = HashMap::new();
                            //println!("draw {pos}");
                            pc.insert(
                                "@pc_section_position".to_string(),
                                (
                                    bytemuck::cast_slice(&rel_pos.to_array()).to_vec(),
                                    ShaderStages::VERTEX,
                                ),
                            );
                            set_push_constants(pipeline_config, &mut render_pass, Some(pc));
                            render_pass.draw_indexed(
                                layer.index_range.clone(),
                                0,
                                layer.vertex_range.start..layer.vertex_range.start + 1,
                            );
                        }
                    }
                }
                "@geo_entities" => {
                    render_pass.set_pipeline(&bound_pipeline.pipeline);

                    let instances = { scene.entity_instances.lock().clone() };

                    for entity_instances in instances.values() {
                        for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                            match bind_group {
                                WmBindGroup::Resource(name) => match &name[..] {
                                    "@bg_entity" => {
                                        render_pass.set_bind_group(
                                            *index,
                                            &entity_instances.uploaded.bind_group,
                                            &[],
                                        );
                                    }
                                    _ => unimplemented!(),
                                },
                                WmBindGroup::Custom(bind_group) => {
                                    render_pass.set_bind_group(*index, bind_group, &[]);
                                }
                            }
                        }

                        let mut pc: HashMap<String, (Vec<u8>, ShaderStages)> = HashMap::new();
                        pc.insert(
                            "@pc_parts_per_entity".to_string(),
                            (
                                bytemuck::cast_slice(&[entity_instances.entity.parts.len() as u32])
                                    .to_vec(),
                                ShaderStages::VERTEX,
                            ),
                        );
                        set_push_constants(pipeline_config, &mut render_pass, Some(pc));

                        render_pass.set_vertex_buffer(0, entity_instances.entity.mesh.slice(..));
                        render_pass
                            .set_vertex_buffer(1, entity_instances.uploaded.instance_vbo.slice(..));

                        render_pass.draw(
                            0..entity_instances.entity.vertex_count,
                            0..entity_instances.capacity,
                        );
                    }
                }
                "@geo_sun_moon" => {
                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Custom(bind_group) => {
                                render_pass.set_bind_group(*index, bind_group, &[]);
                            }
                            WmBindGroup::Resource(_) => {}
                        }
                    }
                    let sun_buffer = wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&SunMoonVertex::load_vertex_sun()),
                        usage: BufferUsages::VERTEX,
                    });
                    let moon_buffer = wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&SunMoonVertex::load_vertex_moon(
                            scene.sky_state.load().moon_phase,
                        )),
                        usage: BufferUsages::VERTEX,
                    });

                    render_pass.set_pipeline(&bound_pipeline.pipeline);
                    let pc = get_environmental_push_constants(scene);
                    set_push_constants(pipeline_config, &mut render_pass, Some(pc));

                    render_pass.set_vertex_buffer(0, sun_buffer.slice(..));
                    render_pass.draw(0..6, 0..1);

                    render_pass.set_vertex_buffer(0, moon_buffer.slice(..));
                    render_pass.draw(0..6, 0..1);
                }
                "@geo_sky_scatter" => {
                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Custom(bind_group) => {
                                render_pass.set_bind_group(*index, bind_group, &[]);
                            }
                            WmBindGroup::Resource(_) => {}
                        }
                    }

                    let (light_sky_vertices, light_sky_indices) =
                        SkyVertex::load_vertex_light_sky();
                    let light_sky_buffer = (
                        wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&light_sky_vertices),
                            usage: BufferUsages::VERTEX,
                        }),
                        wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&light_sky_indices),
                            usage: BufferUsages::INDEX,
                        }),
                    );

                    let (dark_sky_vertices, dark_sky_indices) = SkyVertex::load_vertex_dark_sky();
                    let dark_sky_buffer = (
                        wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&dark_sky_vertices),
                            usage: BufferUsages::VERTEX,
                        }),
                        wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&dark_sky_indices),
                            usage: BufferUsages::INDEX,
                        }),
                    );

                    render_pass.set_pipeline(&bound_pipeline.pipeline);
                    let pc = get_environmental_push_constants(scene);
                    set_push_constants(pipeline_config, &mut render_pass, Some(pc));

                    render_pass.set_vertex_buffer(0, light_sky_buffer.0.slice(..));
                    render_pass.set_index_buffer(light_sky_buffer.1.slice(..), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..24, 0, 0..1);

                    render_pass.set_vertex_buffer(0, dark_sky_buffer.0.slice(..));
                    render_pass.set_index_buffer(dark_sky_buffer.1.slice(..), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..24, 0, 0..1);
                }
                "@geo_sky_fog" => {
                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Custom(bind_group) => {
                                render_pass.set_bind_group(*index, bind_group, &[]);
                            }
                            WmBindGroup::Resource(_) => {}
                        }
                    }

                    let (fog_sphere_vertices, fog_sphere_indices) = SkyVertex::load_fog_sphere();
                    let fog_sphere = (
                        wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&fog_sphere_vertices),
                            usage: BufferUsages::VERTEX,
                        }),
                        wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&fog_sphere_indices),
                            usage: BufferUsages::INDEX,
                        }),
                    );

                    render_pass.set_pipeline(&bound_pipeline.pipeline);
                    let pc = get_environmental_push_constants(scene);
                    set_push_constants(pipeline_config, &mut render_pass, Some(pc));

                    render_pass.set_vertex_buffer(0, fog_sphere.0.slice(..));
                    render_pass.set_index_buffer(fog_sphere.1.slice(..), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..51, 0, 0..1);
                }
                // "@geo_sky_stars" => {
                //     for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                //         match bind_group {
                //             WmBindGroup::Custom(bind_group) => {
                //                 render_pass.set_bind_group(*index, bind_group, &[]);
                //             }
                //             WmBindGroup::Resource(_) => {}
                //         }
                //     }
                //     let stars_vertex_buffer = scene.stars_vertex_buffer.read();
                //     let stars_vertex = stars_vertex_buffer.as_ref().unwrap().slice(..);
                // 
                //     let stars_index_buffer = scene.stars_index_buffer.read();
                //     let stars_index = stars_index_buffer.as_ref().unwrap().slice(..);
                // 
                //     render_pass.set_pipeline(&bound_pipeline.pipeline);
                //     let pc = get_environmental_push_constants(scene);
                //     set_push_constants(pipeline_config, &mut render_pass, Some(pc));
                // 
                //     render_pass.set_vertex_buffer(0, stars_vertex);
                //     render_pass.set_index_buffer(stars_index, IndexFormat::Uint32);
                //     render_pass.draw_indexed(0..*scene.stars_length.read(), 0, 0..1);
                // }
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

fn get_environmental_push_constants(scene: &Scene) -> HashMap<String, (Vec<u8>, ShaderStages)> {
    let sky = &scene.sky_state.load();
    let render_effects = &scene.render_effects.load();

    let mut pc: HashMap<String, (Vec<u8>, ShaderStages)> = HashMap::new();
    pc.insert(
        "@pc_environment_data".to_string(),
        (
            bytemuck::cast_slice(&[
                sky.angle,
                sky.brightness,
                sky.star_shimmer,
                render_effects.fog_start,
                render_effects.fog_end,
                render_effects.fog_shape,
                render_effects.fog_color[0],
                render_effects.fog_color[1],
                render_effects.fog_color[2],
                render_effects.fog_color[3],
                sky.color[0],
                sky.color[1],
                sky.color[2],
                render_effects.dimension_fog_color[0],
                render_effects.dimension_fog_color[1],
                render_effects.dimension_fog_color[2],
                render_effects.dimension_fog_color[3],
            ])
            .to_vec(),
            ShaderStages::VERTEX_FRAGMENT,
        ),
    );
    pc
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
                .and_then(|others| others.get(resource))
            {
                None => unimplemented!("Unknown push constant resource value"),
                Some((data, stages)) => {
                    render_pass.set_push_constants(*stages, *offset as u32, data)
                }
            }
        });
}
