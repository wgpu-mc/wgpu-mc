//! # General
//!
//! This is about the rendering pipeline, and implements the logic behind
//! [shaderpack::ShaderPackConfig].

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use arc_swap::ArcSwap;
use cgmath::{Matrix3, Matrix4, SquareMatrix};
use parking_lot::RwLock;
use treeculler::{BVol, Frustum, Vec3, AABB};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, ColorTargetState, CommandEncoderDescriptor, DepthStencilState, Face,
    FragmentState, FrontFace, LoadOp, Operations, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, PushConstantRange, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, StoreOp, SurfaceConfiguration, TextureFormat,
    VertexBufferLayout, VertexState,
};

use crate::mc::chunk::{Chunk, ChunkBuffers, ChunkPos, CHUNK_SECTION_HEIGHT, SECTIONS_PER_CHUNK};
use crate::mc::resource::ResourcePath;
use crate::render::pipeline::{QuadVertex, BLOCK_ATLAS};
use crate::render::shader::WgslShader;
use crate::render::shaderpack::{
    LonghandResourceConfig, Mat3ValueOrMult, Mat4ValueOrMult, PipelineConfig, ShaderPackConfig,
    ShorthandResourceConfig, TypeResourceConfig,
};
use crate::texture::{BindableTexture, TextureHandle};
use crate::util::{BindableBuffer, WmArena};
use crate::WmRenderer;

pub struct GeometryInfo<'pass, 'resource: 'pass, 'renderer> {
    pub wm: &'renderer WmRenderer,
    pub render_pass: &'renderer mut RenderPass<'pass>,
    pub graph: &'pass ShaderGraph,
    pub config: &'renderer PipelineConfig,
    pub resources: &'resource HashMap<String, CustomResource>,
    pub arena: &'resource WmArena<'resource>,
    pub surface_config: &'renderer SurfaceConfiguration,
    pub chunk_offset: ChunkPos,
}

pub trait GeometryCallback: Send + Sync {
    fn render(&self, info: GeometryInfo);
}

fn mat3_update(
    resource: &CustomResource,
    wm: &WmRenderer,
    resources: &HashMap<String, CustomResource>,
) {
    let mut mat3 = Matrix3::<f32>::identity();

    if let ResourceInternal::Mat3(Mat3ValueOrMult::Mult { mult }, lock, ssbo) = &*resource.data {
        mult.iter().for_each(|mat_name| {
            let resource = resources.get(mat_name).unwrap();

            match &*resource.data {
                ResourceInternal::Mat3(_, lock, _) => {
                    mat3 = mat3 * (*lock.read());
                },
                _ => panic!("Invalid config. Mat3 resource multiplication should only ever refer to other Mat3s")
            }
        });

        *lock.write() = mat3;

        let mat3_array: [[f32; 3]; 3] = mat3.into();

        wm.wgpu_state
            .queue
            .write_buffer(&ssbo.buffer, 0, bytemuck::cast_slice(&mat3_array));
    }
}

fn mat4_update(
    resource: &CustomResource,
    wm: &WmRenderer,
    resources: &HashMap<String, CustomResource>,
) {
    let mut mat4 = Matrix4::<f32>::identity();

    if let ResourceInternal::Mat4(Mat4ValueOrMult::Mult { mult }, lock, ssbo) = &*resource.data {
        mult.iter().for_each(|mat_name| {
            {
                let resource = resources.get(mat_name).expect(mat_name);
                match &*resource.data {
                    ResourceInternal::Mat4(_, lock, _) => {
                        mat4 = mat4 * (*lock.read());
                    },
                    _ => panic!("Invalid config. Mat4 resource multiplication should only ever refer to other Mat4s")
                }
            };
        });

        *lock.write() = mat4;

        let mat4_array: [[f32; 4]; 4] = mat4.into();

        wm.wgpu_state
            .queue
            .write_buffer(&ssbo.buffer, 0, bytemuck::cast_slice(&mat4_array));
    }
}

#[derive(Debug)]
pub enum TextureResource {
    Handle(TextureHandle),
    Bindable(Arc<ArcSwap<BindableTexture>>),
}

#[derive(Debug)]
pub enum ResourceInternal {
    Texture(TextureResource, bool),
    Blob(BindableBuffer),
    Mat3(
        Mat3ValueOrMult,
        Arc<RwLock<Matrix3<f32>>>,
        Arc<BindableBuffer>,
    ),
    Mat4(
        Mat4ValueOrMult,
        Arc<RwLock<Matrix4<f32>>>,
        Arc<BindableBuffer>,
    ),
    F32(f32, BindableBuffer),
    F64(f64, BindableBuffer),
    U32(u32, BindableBuffer),
    I32(i32, BindableBuffer),
    I64(i64, BindableBuffer),
}

type UpdateCallback = fn(&CustomResource, &WmRenderer, &HashMap<String, CustomResource>);

pub struct CustomResource {
    //If this resource is updated each frame, this is what needs to be called
    pub update: Option<UpdateCallback>,
    pub data: Arc<ResourceInternal>,
}

impl CustomResource {
    pub fn get_mat4(&self) -> Option<Matrix4<f32>> {
        if let ResourceInternal::Mat4(_, lock, _) = &*self.data {
            Some(*lock.read())
        } else {
            None
        }
    }
}

/// This struct holds information on the entirety of the rendering pipeline.
pub struct ShaderGraph {
    pub pack: ShaderPackConfig,
    pub pipelines: HashMap<String, RenderPipeline>,
    pub resources: HashMap<String, CustomResource>,
    pub geometry: HashMap<String, Box<dyn GeometryCallback>>,
    quad: Option<wgpu::Buffer>,
    query_results: Option<wgpu::Buffer>,
}

impl ShaderGraph {
    pub fn new(
        pack: ShaderPackConfig,
        resources: HashMap<String, CustomResource>,
        geometry: HashMap<String, Box<dyn GeometryCallback>>,
    ) -> Self {
        Self {
            pack,
            pipelines: HashMap::new(),
            resources,
            geometry,
            quad: None,
            query_results: None,
        }
    }

    pub fn init(
        &mut self,
        wm: &WmRenderer,
        resource_types: Option<&HashMap<String, String>>,
        mut additional_geometry: Option<HashMap<String, VertexBufferLayout>>,
    ) {
        let mut resource_types = resource_types.cloned().unwrap_or(HashMap::new());

        resource_types.insert("wm_ssbo_chunk_vertices".into(), "ssbo".into());
        resource_types.insert("wm_ssbo_chunk_indices".into(), "ssbo".into());

        let mut resources = HashMap::new();

        self.query_results = Some(
            wm.wgpu_state
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: &[0; 1024],
                    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                }),
        );

        self.quad = Some(
            wm.wgpu_state
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        -1.0f32, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
                    ]),
                    usage: BufferUsages::VERTEX,
                }),
        );

        let block_atlas = wm
            .mc
            .texture_manager
            .atlases
            .load()
            .get(BLOCK_ATLAS)
            .unwrap()
            .load();

        resources.insert(
            "wm_texture_atlas_blocks".into(),
            CustomResource {
                update: None,
                data: Arc::new(ResourceInternal::Texture(
                    TextureResource::Bindable(block_atlas.bindable_texture.clone()),
                    false,
                )),
            },
        );

        for (resource_id, definition) in &self.pack.resources.resources {
            let resource_id = resource_id.clone();

            Self::insert_resources(&wm, &mut resources, definition, resource_id);
        }

        let pipelines = wm.pipelines.load();
        let layouts = pipelines.bind_group_layouts.read();

        self.pack
            .pipelines
            .pipelines
            .iter()
            .for_each(|(name, definition)| match &self.pack.support[..] {
                "wgsl" => {
                    let shader = WgslShader::init(
                        &ResourcePath(format!("wgpu_mc:shaders/{name}.wgsl")),
                        &*wm.mc.resource_provider,
                        &wm.wgpu_state.device,
                        "frag".into(),
                        "vert".into(),
                    )
                    .unwrap();

                    let pipeline_layout =
                        wm.wgpu_state
                            .device
                            .create_pipeline_layout(&PipelineLayoutDescriptor {
                                label: None,
                                bind_group_layouts: &definition
                                    .uniforms
                                    .iter()
                                    .map(|(_index, uniform)| {
                                        if let Some(resource) = resources.get(uniform) {
                                            match &*resource.data {
                                                ResourceInternal::Texture(_, depth) => layouts
                                                    .get(if *depth {
                                                        "texture_depth"
                                                    } else {
                                                        "texture"
                                                    })
                                                    .unwrap(),
                                                ResourceInternal::Mat3(..)
                                                | ResourceInternal::Mat4(..) => {
                                                    layouts.get("matrix").unwrap()
                                                }
                                                ResourceInternal::Blob(..)
                                                | ResourceInternal::F32(..)
                                                | ResourceInternal::F64(..)
                                                | ResourceInternal::U32(..)
                                                | ResourceInternal::I32(..)
                                                | ResourceInternal::I64(..) => {
                                                    layouts.get("ssbo").unwrap()
                                                }
                                            }
                                        } else {
                                            layouts
                                                .get(resource_types.get(uniform).expect(uniform))
                                                .unwrap()
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                                push_constant_ranges: &definition
                                    .push_constants
                                    .iter()
                                    .map(|(offset, resource)| match &resource[..] {
                                        "wm_pc_chunk_position" => PushConstantRange {
                                            stages: ShaderStages::VERTEX,
                                            range: *offset as u32..*offset as u32 + 12,
                                        },
                                        "wm_pc_framebuffer_size" => PushConstantRange {
                                            stages: ShaderStages::FRAGMENT,
                                            range: *offset as u32..*offset as u32 + 8,
                                        },
                                        _ => unimplemented!("Unknown push constant resource value"),
                                    })
                                    .collect::<Vec<_>>(),
                            });

                    let vertex_buffer = match &definition.geometry[..] {
                        "wm_geo_terrain" => None,
                        "wm_geo_quad" => Some([QuadVertex::desc()]),
                        _ => {
                            if let Some(additional_geometry) = &mut additional_geometry {
                                Some([additional_geometry.remove(&definition.geometry).unwrap()])
                            } else {
                                unimplemented!("Unknown geometry");
                            }
                        }
                    };

                    let pipeline =
                        wm.wgpu_state
                            .device
                            .create_render_pipeline(&RenderPipelineDescriptor {
                                label: None,
                                layout: Some(&pipeline_layout),
                                vertex: VertexState {
                                    module: &shader.shader,
                                    entry_point: "vert",
                                    buffers: match &vertex_buffer {
                                        None => &[],
                                        Some(buffer) => buffer,
                                    },
                                },
                                primitive: PrimitiveState {
                                    topology: PrimitiveTopology::TriangleList,
                                    strip_index_format: None,
                                    front_face: FrontFace::Ccw,
                                    cull_mode: Some(Face::Back),
                                    // cull_mode: None,
                                    unclipped_depth: false,
                                    polygon_mode: PolygonMode::Fill,
                                    conservative: false,
                                },
                                depth_stencil: definition.depth.as_ref().map(|_| {
                                    DepthStencilState {
                                        format: TextureFormat::Depth32Float,
                                        depth_write_enabled: true,
                                        depth_compare: wgpu::CompareFunction::Less,
                                        stencil: Default::default(),
                                        bias: Default::default(),
                                    }
                                }),
                                multisample: Default::default(),
                                fragment: Some(FragmentState {
                                    module: &shader.shader,
                                    entry_point: "frag",
                                    targets: &definition
                                        .output
                                        .iter()
                                        .map(|_| {
                                            Some(ColorTargetState {
                                            format: TextureFormat::Bgra8Unorm,
                                            blend: Some(match &definition.blending[..] {
                                                "alpha_blending" => {
                                                    wgpu::BlendState::ALPHA_BLENDING
                                                }
                                                "premultiplied_alpha_blending" => {
                                                    wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING
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

                    self.pipelines.insert(name.clone(), pipeline);
                }
                "glsl" => todo!(),
                _ => unimplemented!("{}", self.pack.support),
            });

        self.resources.extend(resources);
    }

    /// Matches on the definition, inserting the resource depending on which variant it is.
    fn insert_resources(
        wm: &&WmRenderer,
        resources: &mut HashMap<String, CustomResource>,
        definition: &ShorthandResourceConfig,
        resource_id: String,
    ) {
        match definition {
            ShorthandResourceConfig::Int(int) => {
                let ssbo = BindableBuffer::new(
                    wm,
                    bytemuck::cast_slice(&[*int]),
                    BufferUsages::STORAGE,
                    "ssbo",
                );

                resources.insert(
                    resource_id,
                    CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::I64(*int, ssbo)),
                    },
                );
            }
            ShorthandResourceConfig::Float(float) => {
                let ssbo = BindableBuffer::new(
                    wm,
                    bytemuck::cast_slice(&[*float]),
                    BufferUsages::UNIFORM,
                    "matrix",
                );

                resources.insert(
                    resource_id,
                    CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::F64(*float, ssbo)),
                    },
                );
            }
            ShorthandResourceConfig::Mat3(mat3) => {
                let ssbo = BindableBuffer::new(
                    wm,
                    bytemuck::cast_slice(&mat3[..]),
                    BufferUsages::UNIFORM,
                    "matrix",
                );

                let matrix3: Matrix3<f32> = (*mat3).into();

                resources.insert(
                    resource_id,
                    CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::Mat3(
                            Mat3ValueOrMult::Value { value: *mat3 },
                            Arc::new(RwLock::new(matrix3)),
                            Arc::new(ssbo),
                        )),
                    },
                );
            }
            ShorthandResourceConfig::Mat4(mat4) => {
                let ssbo = BindableBuffer::new(
                    wm,
                    bytemuck::cast_slice(&mat4[..]),
                    BufferUsages::UNIFORM,
                    "matrix",
                );

                let matrix4: Matrix4<f32> = (*mat4).into();

                resources.insert(
                    resource_id,
                    CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::Mat4(
                            Mat4ValueOrMult::Value { value: *mat4 },
                            Arc::new(RwLock::new(matrix4)),
                            Arc::new(ssbo),
                        )),
                    },
                );
            }
            ShorthandResourceConfig::Longhand(longhand) => match &longhand.typed {
                TypeResourceConfig::Texture3d { .. } => todo!(),
                TypeResourceConfig::Texture2d { src, .. } => {
                    if !src.is_empty() {
                        todo!()
                    } else {
                        let handle = wm.create_texture_handle(
                            resource_id.clone(),
                            TextureFormat::Bgra8Unorm,
                            &wm.wgpu_state.surface.read().1,
                        );
                        resources.insert(
                            resource_id,
                            CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::Texture(
                                    TextureResource::Handle(handle),
                                    false,
                                )),
                            },
                        );
                    }
                }
                TypeResourceConfig::TextureDepth { .. } => {
                    let handle = wm.create_texture_handle(
                        resource_id.clone(),
                        TextureFormat::Depth32Float,
                        &wm.wgpu_state.surface.read().1,
                    );
                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::Texture(
                                TextureResource::Handle(handle),
                                true,
                            )),
                        },
                    );
                }
                TypeResourceConfig::F32 { value, .. } => {
                    let ssbo = BindableBuffer::new(
                        wm,
                        bytemuck::cast_slice(&[*value]),
                        BufferUsages::STORAGE,
                        "ssbo",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::F32(*value, ssbo)),
                        },
                    );
                }
                TypeResourceConfig::F64 { value, .. } => {
                    let ssbo = BindableBuffer::new(
                        wm,
                        bytemuck::cast_slice(&[*value]),
                        BufferUsages::STORAGE,
                        "ssbo",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::F64(*value, ssbo)),
                        },
                    );
                }
                TypeResourceConfig::I64 { value, .. } => {
                    let ssbo = BindableBuffer::new(
                        wm,
                        bytemuck::cast_slice(&[*value]),
                        BufferUsages::STORAGE,
                        "ssbo",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::I64(*value, ssbo)),
                        },
                    );
                }
                TypeResourceConfig::I32 { value, .. } => {
                    let ssbo = BindableBuffer::new(
                        wm,
                        bytemuck::cast_slice(&[*value]),
                        BufferUsages::STORAGE,
                        "ssbo",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::I32(*value, ssbo)),
                        },
                    );
                }
                TypeResourceConfig::Mat3(mat3) => {
                    let value = match mat3 {
                        Mat3ValueOrMult::Value { value } => *value,
                        Mat3ValueOrMult::Mult { .. } => [[0.0; 3]; 3],
                    };

                    let ssbo = BindableBuffer::new(
                        wm,
                        bytemuck::cast_slice(&value),
                        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        "matrix",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: match mat3 {
                                Mat3ValueOrMult::Value { .. } => None,
                                Mat3ValueOrMult::Mult { .. } => Some(mat3_update),
                            },
                            data: Arc::new(ResourceInternal::Mat3(
                                mat3.clone(),
                                Arc::new(RwLock::new(value.into())),
                                Arc::new(ssbo),
                            )),
                        },
                    );
                }
                TypeResourceConfig::Mat4(mat4) => {
                    let value = match mat4 {
                        Mat4ValueOrMult::Value { value } => *value,
                        Mat4ValueOrMult::Mult { .. } => [[0.0; 4]; 4],
                    };

                    let ssbo = BindableBuffer::new(
                        wm,
                        bytemuck::cast_slice(&value),
                        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        "matrix",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: match mat4 {
                                Mat4ValueOrMult::Value { .. } => None,
                                Mat4ValueOrMult::Mult { .. } => Some(mat4_update),
                            },
                            data: Arc::new(ResourceInternal::Mat4(
                                mat4.clone(),
                                Arc::new(RwLock::new(value.into())),
                                Arc::new(ssbo),
                            )),
                        },
                    );
                }
                TypeResourceConfig::Blob { .. } => todo!(),
            },
        }
    }

    pub fn render<'graph, 'resource: 'graph, 'a, 'b, 'c: 'b>(
        &'graph self,
        wm: &WmRenderer,
        output_texture: &'graph wgpu::TextureView,
        surface_config: &SurfaceConfiguration,
    ) {
        let arena = WmArena::new(1024);

        let mut encoder = wm
            .wgpu_state
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        self.resources
            .iter()
            .for_each(|(_, resource)| match resource.update {
                None => {}
                Some(func) => func(resource, wm, &self.resources),
            });

        let resource_borrow = self.resources.iter().collect();

        let texture_handles = wm.texture_handles.read();

        //The first render pass that uses the framebuffer's depth buffer should clear it
        let mut should_clear_depth = true;

        let chunk_offset = *wm.mc.chunks.chunk_offset.lock();

        let projection_matrix = self
            .resources
            .get("wm_mat4_projection")
            .unwrap()
            .get_mat4()
            .unwrap();
        let view_matrix = self
            .resources
            .get("wm_mat4_view")
            .unwrap()
            .get_mat4()
            .unwrap();

        let frustum = Frustum::from_modelview_projection((projection_matrix * view_matrix).into());

        for (_query_index, (name, config)) in
            (&self.pack.pipelines.pipelines).into_iter().enumerate()
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None, // TODO maybe use the pipeline name or something? for easier debugging if needed
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &config
                    .output
                    .iter()
                    .map(|texture_name| {
                        let resource_definition = self.pack.resources.resources.get(texture_name);

                        //TODO: If the texture resource is defined as being cleared after each frame. Should use a HashMap to replace the should_clear_depth variable
                        let _clear = matches!(
                            resource_definition,
                            Some(&ShorthandResourceConfig::Longhand(LonghandResourceConfig {
                                typed: TypeResourceConfig::Texture2d {
                                    clear_after_frame: true,
                                    ..
                                },
                                ..
                            }))
                        );

                        Some(RenderPassColorAttachment {
                            view: match &texture_name[..] {
                                "wm_framebuffer_texture" => output_texture,
                                name => {
                                    &arena
                                        .alloc(
                                            texture_handles
                                                .get(name)
                                                .unwrap()
                                                .bindable_texture
                                                .load(),
                                        )
                                        .tsv
                                        .view
                                }
                            },
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: StoreOp::Store,
                            },
                        })
                    })
                    .collect::<Vec<_>>(),
                depth_stencil_attachment: config.depth.as_ref().map(|depth_texture| {
                    let will_clear_depth = should_clear_depth;
                    should_clear_depth = false;

                    RenderPassDepthStencilAttachment {
                        view: &arena
                            .alloc(
                                texture_handles
                                    .get(depth_texture)
                                    .unwrap()
                                    .bindable_texture
                                    .load(),
                            )
                            .tsv
                            .view,
                        depth_ops: Some(Operations {
                            // load: if will_clear_depth { LoadOp::Clear(1.0) } else { LoadOp::Load },
                            load: LoadOp::Clear(1.0),
                            store: if will_clear_depth {
                                StoreOp::Store
                            } else {
                                StoreOp::Discard
                            },
                        }),
                        stencil_ops: None,
                    }
                }),
            });

            render_pass.set_pipeline(self.pipelines.get(name).unwrap());

            match &config.geometry[..] {
                "wm_geo_terrain" => {
                    let layers = wm.pipelines.load().chunk_layers.load();
                    let chunks = wm.mc.chunks.loaded_chunks.read();

                    for layer in &**layers {
                        for (_pos, chunk_swap) in chunks.iter() {
                            let chunk = arena.alloc(chunk_swap.load());
                            let buffers = arena.alloc(chunk.buffers.load_full());
                            let sections = chunk.sections.read();

                            for section_index in 0..SECTIONS_PER_CHUNK {
                                let min = Vec3::new(
                                    ((chunk.pos[0] + chunk_offset[0]) * 16) as f32,
                                    (section_index * CHUNK_SECTION_HEIGHT) as f32,
                                    ((chunk.pos[1] + chunk_offset[1]) * 16) as f32,
                                );
                                let max = min
                                    + Vec3::new(
                                        16.0,
                                        ((section_index + 1) * CHUNK_SECTION_HEIGHT) as f32,
                                        16.0,
                                    );

                                let aabb = AABB::<f32>::new(min, max);

                                if aabb.test_against_frustum(&frustum, 0) == u8::MAX {
                                    continue;
                                }

                                let baked_layer = match sections.get(layer.name()) {
                                    None => continue,
                                    Some(section) => &section[section_index],
                                };

                                bind_uniforms(
                                    config,
                                    &resource_borrow,
                                    &arena,
                                    &mut render_pass,
                                    (*buffers).as_ref().as_ref(),
                                );
                                set_push_constants(
                                    config,
                                    &mut render_pass,
                                    Some(chunk),
                                    surface_config,
                                    chunk_offset,
                                    section_index,
                                );

                                render_pass.draw(baked_layer.clone(), 0..1);
                            }
                        }
                    }
                }
                "wm_geo_entities" | "wm_geo_transparent" | "wm_geo_fluid" | "wm_geo_skybox"
                | "wm_geo_quad" => {
                    bind_uniforms(config, &resource_borrow, &arena, &mut render_pass, None);
                    set_push_constants(
                        config,
                        &mut render_pass,
                        None,
                        surface_config,
                        chunk_offset,
                        0,
                    );

                    render_pass.set_pipeline(self.pipelines.get(name).unwrap());
                    render_pass.set_vertex_buffer(0, self.quad.as_ref().unwrap().slice(..));
                    render_pass.draw(0..6, 0..1);
                }

                _ => {
                    if let Some(geo) = self.geometry.get(&config.geometry) {
                        render_pass.set_pipeline(self.pipelines.get(name).unwrap());
                        geo.render(GeometryInfo {
                            wm,
                            render_pass: &mut render_pass,
                            graph: self,
                            config,
                            resources: &self.resources,
                            arena: &arena,
                            surface_config,
                            chunk_offset,
                        });
                    } else {
                        unimplemented!("Unknown geometry {}", &config.geometry);
                    }
                }
            };
        }

        wm.wgpu_state.queue.submit([encoder.finish()]);
    }
}

pub fn bind_uniforms<'resource: 'pass, 'pass>(
    config: &PipelineConfig,
    resources: &'resource HashMap<&String, &'resource CustomResource>,
    arena: &WmArena<'pass>,
    render_pass: &mut RenderPass<'pass>,
    chunk_buffers: Option<&'pass ChunkBuffers>,
) {
    for (index, resource_name) in &config.uniforms {
        match &resource_name[..] {
            "wm_ssbo_chunk_vertices" => {
                render_pass.set_bind_group(
                    *index as u32,
                    &chunk_buffers.unwrap().vertex_buffer.bind_group,
                    &[],
                );
                continue;
            }
            "wm_ssbo_chunk_indices" => {
                render_pass.set_bind_group(
                    *index as u32,
                    &chunk_buffers.unwrap().index_buffer.bind_group,
                    &[],
                );
                continue;
            }
            _ => {}
        }

        let bind_group = match &*resources.get(resource_name).unwrap().data {
            ResourceInternal::Texture(handle, _) => match handle {
                TextureResource::Handle(handle) => {
                    &arena.alloc(handle.bindable_texture.load()).bind_group
                }
                TextureResource::Bindable(bindable) => &arena.alloc(bindable.load()).bind_group,
            },
            ResourceInternal::Blob(BindableBuffer { bind_group, .. }) => bind_group,
            ResourceInternal::Mat3(_, _, bindable) | ResourceInternal::Mat4(_, _, bindable) => {
                &bindable.bind_group
            }
            ResourceInternal::F32(_, BindableBuffer { bind_group, .. })
            | ResourceInternal::F64(_, BindableBuffer { bind_group, .. })
            | ResourceInternal::U32(_, BindableBuffer { bind_group, .. })
            | ResourceInternal::I32(_, BindableBuffer { bind_group, .. })
            | ResourceInternal::I64(_, BindableBuffer { bind_group, .. }) => bind_group,
        };

        render_pass.set_bind_group(*index as u32, bind_group, &[]);
    }
}

pub fn set_push_constants(
    pipeline: &PipelineConfig,
    render_pass: &mut RenderPass,
    chunk: Option<&Chunk>,
    surface_config: &SurfaceConfiguration,
    chunk_offset: ChunkPos,
    section_y: usize,
) {
    pipeline
        .push_constants
        .iter()
        .for_each(|(offset, resource)| match &resource[..] {
            "wm_pc_framebuffer_size" => {
                render_pass.set_push_constants(
                    ShaderStages::FRAGMENT,
                    *offset as u32,
                    bytemuck::cast_slice(&[
                        surface_config.width as f32,
                        surface_config.height as f32,
                    ]),
                );
            }
            "wm_pc_chunk_position" => render_pass.set_push_constants(
                ShaderStages::VERTEX,
                *offset as u32,
                bytemuck::cast_slice(&[
                    chunk.unwrap().pos[0] + chunk_offset[0],
                    section_y as i32,
                    chunk.unwrap().pos[1] + chunk_offset[1],
                ]),
            ),
            _ => unimplemented!("Unknown push constant resource value"),
        });
}
