use arc_swap::ArcSwap;
use cgmath::{Matrix3, Matrix4, SquareMatrix};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
use treeculler::{BVol, Frustum, Vec3, AABB};

use crate::mc::chunk::{Chunk, ChunkPos};
use crate::mc::resource::ResourcePath;
use crate::render::pipeline::{QuadVertex, Vertex, BLOCK_ATLAS};
use crate::render::shader::WgslShader;
use crate::render::shaderpack::{
    LonghandResourceConfig, Mat3ValueOrMult, Mat4ValueOrMult, PipelineConfig, ShaderPackConfig,
    ShorthandResourceConfig, TypeResourceConfig,
};
use crate::texture::{BindableTexture, TextureHandle};
use crate::util::{BindableBuffer, WmArena};
use crate::WmRenderer;

use crate::mc::entity::{EntityInstances, InstanceVertex};
use crate::render::entity::EntityVertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, ColorTargetState, CommandEncoderDescriptor, DepthStencilState, FragmentState,
    LoadOp, Operations, PipelineLayoutDescriptor, PushConstantRange, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, SurfaceConfiguration, TextureFormat,
    VertexBufferLayout, VertexState,
};

pub trait GeometryCallback: Send + Sync {
    fn render<'pass, 'resource: 'pass>(
        &self,
        wm: &WmRenderer,
        pass: &mut RenderPass<'pass>,
        graph: &'pass ShaderGraph,
        config: &PipelineConfig,
        resources: &'resource HashMap<String, CustomResource>,
        arena: &'resource WmArena<'resource>,
        surface_config: &SurfaceConfiguration,
        chunk_offset: ChunkPos,
    );
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
            match &mat_name[..] {
                _ => {
                    let resource = resources.get(mat_name).expect(mat_name);
                    match &*resource.data {
                        ResourceInternal::Mat4(_, lock, _) => {
                            mat4 = mat4 * (*lock.read());
                        },
                        _ => panic!("Invalid config. Mat4 resource multiplication should only ever refer to other Mat4s")
                    }
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
    Blob(Arc<BindableBuffer>),
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

pub struct CustomResource {
    //If this resource is updated each frame, this is what needs to be called
    pub update: Option<fn(&Self, &WmRenderer, &HashMap<String, CustomResource>)>,
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

pub struct ShaderGraph {
    pub pack: ShaderPackConfig,
    pub pipelines: HashMap<String, RenderPipeline>,
    pub resources: HashMap<String, CustomResource>,
    pub geometry: HashMap<String, Box<dyn GeometryCallback>>,
    quad: Option<wgpu::Buffer>,
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
        }
    }

    pub fn init(
        &mut self,
        wm: &WmRenderer,
        resource_types: Option<&HashMap<String, String>>,
        mut additional_geometry: Option<HashMap<String, Vec<VertexBufferLayout>>>,
    ) {
        let mut resource_types = match resource_types {
            None => HashMap::new(),
            Some(map) => map.clone(),
        };

        resource_types.insert("wm_ssbo_entity_part_transforms".into(), "ssbo".into());
        resource_types.insert("wm_texture_entities".into(), "texture".into());

        let mut resources = HashMap::new();

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
                                            range: *offset as u32..*offset as u32 + 8,
                                        },
                                        "wm_pc_framebuffer_size" => PushConstantRange {
                                            stages: ShaderStages::FRAGMENT,
                                            range: *offset as u32..*offset as u32 + 8,
                                        },
                                        _ => unimplemented!("Unknown push constant resource value"),
                                    })
                                    .collect::<Vec<_>>(),
                            });

                    let buffers = match &definition.geometry[..] {
                        "wm_geo_terrain" => vec![Vertex::desc()],
                        "wm_geo_quad" => vec![QuadVertex::desc()],
                        "wm_geo_entities" => vec![EntityVertex::desc(), InstanceVertex::desc()],
                        _ => {
                            if let Some(additional_geometry) = &mut additional_geometry {
                                additional_geometry
                                    .remove(&definition.geometry)
                                    .unwrap()
                                    .clone()
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
                                    buffers: &buffers,
                                },
                                primitive: Default::default(),
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

        self.resources.extend(resources.into_iter());
    }

    pub fn render<'graph, 'resource: 'graph, 'a, 'b, 'c: 'b>(
        &'graph self,
        wm: &WmRenderer,
        output_texture: &'graph wgpu::TextureView,
        surface_config: &SurfaceConfiguration,
        entity_instances: &HashMap<String, EntityInstances>,
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

        let _chunk_offset = *wm.mc.chunks.chunk_offset.lock();

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

        for (name, config) in &self.pack.pipelines.pipelines {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &config
                    .output
                    .iter()
                    .map(|texture_name| {
                        let resource_definition = self.pack.resources.resources.get(texture_name);

                        //TODO: If the texture resource is defined as being cleared after each frame. Should use a HashMap to replace the should_clear_depth variable
                        let _clear = match resource_definition {
                            Some(&ShorthandResourceConfig::Longhand(LonghandResourceConfig {
                                typed:
                                    TypeResourceConfig::Texture2d {
                                        clear_after_frame: true,
                                        ..
                                    },
                                ..
                            })) => true,
                            _ => false,
                        };

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
                                store: true,
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
                            store: will_clear_depth,
                        }),
                        stencil_ops: None,
                    }
                }),
            });

            let chunk_offset = [0, 0];

            render_pass.set_pipeline(self.pipelines.get(name).unwrap());

            match &config.geometry[..] {
                "wm_geo_terrain" => {
                    let layers = wm.pipelines.load().chunk_layers.load();
                    let chunks = wm.mc.chunks.loaded_chunks.read();

                    for layer in &**layers {
                        for (_pos, chunk_swap) in &*chunks {
                            let chunk = arena.alloc(chunk_swap.load());

                            let min = Vec3::new(
                                (chunk.pos[0] * 16) as f32,
                                0.0,
                                (chunk.pos[1] * 16) as f32,
                            );
                            let max = min + Vec3::new(16.0, 384.0, 16.0);

                            let aabb = AABB::<f32>::new(min, max);

                            if aabb.test_against_frustum(&frustum, 0) == u8::MAX {
                                continue;
                            }

                            let (chunk_vbo, verts) =
                                match arena.alloc(chunk.baked_layers.read()).get(layer.name()) {
                                    None => continue,
                                    Some(tuple) => tuple,
                                };

                            bind_uniforms(config, &resource_borrow, &arena, &mut render_pass);
                            set_push_constants(
                                config,
                                &mut render_pass,
                                Some(&chunk),
                                surface_config,
                                chunk_offset,
                            );

                            render_pass.set_vertex_buffer(0, chunk_vbo.slice(..));
                            render_pass.draw(0..verts.len() as u32, 0..1);
                        }
                    }
                }
                "wm_geo_quad" => {
                    bind_uniforms(config, &resource_borrow, &arena, &mut render_pass);
                    set_push_constants(
                        config,
                        &mut render_pass,
                        None,
                        surface_config,
                        chunk_offset,
                    );

                    render_pass.set_pipeline(self.pipelines.get(name).unwrap());
                    render_pass.set_vertex_buffer(0, self.quad.as_ref().unwrap().slice(..));
                    render_pass.draw(0..6, 0..1);
                }
                "wm_geo_entities" => {
                    let entities = wm.mc.entity_models.read();

                    entities.iter().for_each(|entity| {
                        let instances = entity_instances.get(&entity.name).unwrap();

                        let read_guard = instances.uploaded.read();
                        let uploaded = read_guard.as_ref().unwrap();

                        let instance_vbo = arena.alloc(uploaded.instance_vbo.clone());
                        let bindable_buffer = uploaded.transform_ssbo.clone();

                        let augmented_resources = resource_borrow
                            .clone()
                            .into_iter()
                            .chain([
                                (
                                    &*arena.alloc("wm_ssbo_entity_part_transforms".into()),
                                    &*arena.alloc(CustomResource {
                                        update: None,
                                        data: Arc::new(ResourceInternal::Blob(bindable_buffer)),
                                    }),
                                ),
                                (
                                    &*arena.alloc("wm_texture_entities".into()),
                                    &*arena.alloc(CustomResource {
                                        update: None,
                                        data: Arc::new(ResourceInternal::Texture(
                                            TextureResource::Bindable(entity.texture.clone()),
                                            false,
                                        )),
                                    }),
                                ),
                            ])
                            .collect();

                        bind_uniforms(
                            config,
                            arena.alloc(augmented_resources),
                            &arena,
                            &mut render_pass,
                        );
                        set_push_constants(
                            config,
                            &mut render_pass,
                            None,
                            surface_config,
                            chunk_offset,
                        );

                        render_pass
                            .set_vertex_buffer(0, arena.alloc(entity.mesh.clone()).slice(..));
                        render_pass.set_vertex_buffer(1, instance_vbo.slice(..));
                        render_pass.draw(0..entity.vertices, 0..instances.instances.len() as u32);
                    });
                }
                "wm_geo_transparent" | "wm_geo_fluid" | "wm_geo_skybox" => todo!(),
                _ => {
                    if let Some(geo) = self.geometry.get(&config.geometry) {
                        render_pass.set_pipeline(self.pipelines.get(name).unwrap());
                        geo.render(
                            wm,
                            &mut render_pass,
                            self,
                            config,
                            &self.resources,
                            &arena,
                            surface_config,
                            chunk_offset,
                        );
                    } else {
                        unimplemented!("Unknown geometry {0}", &config.geometry);
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
    arena: &WmArena<'resource>,
    render_pass: &mut RenderPass<'pass>,
) {
    for (index, resource_name) in &config.uniforms {
        let bind_group = match &*resources.get(resource_name).unwrap().data {
            ResourceInternal::Texture(handle, _) => match handle {
                TextureResource::Handle(handle) => {
                    &arena.alloc(handle.bindable_texture.load()).bind_group
                }
                TextureResource::Bindable(bindable) => &arena.alloc(bindable.load()).bind_group,
            },
            ResourceInternal::Blob(bindable) => &bindable.bind_group,
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
                    chunk.unwrap().pos[0] - chunk_offset[0],
                    chunk.unwrap().pos[1] - chunk_offset[1],
                ]),
            ),
            _ => unimplemented!("Unknown push constant resource value"),
        });
}
