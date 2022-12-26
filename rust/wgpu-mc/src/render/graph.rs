use bytemuck::Pod;
use cgmath::{Matrix3, Matrix4, SquareMatrix};
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;

use crate::mc::resource::ResourcePath;
use crate::render::pipeline::{BLOCK_ATLAS, Vertex};
use crate::render::shader::WgslShader;
use crate::render::shaderpack::{
    LonghandResourceConfig, Mat3ValueOrMult, Mat4ValueOrMult, ShaderPackConfig,
    ShorthandResourceConfig, TypeResourceConfig,
};
use crate::texture::{BindableTexture, TextureHandle, TextureSamplerView};
use crate::util::{WmArena, UniformStorage};
use crate::WmRenderer;
use serde::Deserialize;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BufferUsages, Color, ColorTargetState,
    CommandEncoderDescriptor, DepthStencilState, FragmentState, LoadOp, Operations,
    PipelineLayoutDescriptor, PushConstantRange, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState,
};

fn mat3_update(
    resource: &CustomResource,
    wm: &WmRenderer,
    resources: &HashMap<String, CustomResource>,
) {
    let mut mat3 = Matrix3::<f32>::identity();

    if let ResourceInternal::Mat3((Mat3ValueOrMult::Mult { mult }, lock, ssbo)) = &*resource.data {
        mult.iter().for_each(|mat_name| {
            let resource = resources.get(mat_name).unwrap();

            match &*resource.data {
                ResourceInternal::Mat3((_, lock, _)) => {
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

    if let ResourceInternal::Mat4((Mat4ValueOrMult::Mult { mult }, lock, ssbo)) = &*resource.data {
        mult.iter().for_each(|mat_name| {
            match &mat_name[..] {
                "wm_projection_mat4" => {
                    mat4 = mat4 * wm.mc.camera.load().build_view_projection_matrix();
                },
                "wm_model_mat4" | "wm_view_mat4" => {},
                _ => {
                    let resource = resources.get(mat_name).unwrap();
                    match &*resource.data {
                        ResourceInternal::Mat4((_, lock, _)) => {
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

pub enum TextureResource {
    Handle(TextureHandle),
    Bindable(Arc<ArcSwap<BindableTexture>>)
}

pub enum ResourceInternal {
    Texture(TextureResource, bool),
    Blob(UniformStorage),
    Mat3((Mat3ValueOrMult, RwLock<Matrix3<f32>>, UniformStorage)),
    Mat4(((Mat4ValueOrMult, RwLock<Matrix4<f32>>, UniformStorage))),
    F32((f32, UniformStorage)),
    F64((f64, UniformStorage)),
    U32((u32, UniformStorage)),
    I32((i32, UniformStorage)),
    I64((i64, UniformStorage)),
}

pub struct CustomResource {
    //If this resource is updated each frame, this is what needs to be called
    pub update: Option<fn(&Self, &WmRenderer, &HashMap<String, CustomResource>)>,
    pub data: Arc<ResourceInternal>,
}

pub struct ShaderGraph {
    pub pack: ShaderPackConfig,
    pub pipelines: HashMap<String, RenderPipeline>,
    pub resources: HashMap<String, CustomResource>,
}

impl ShaderGraph {
    pub fn new(pack: ShaderPackConfig) -> Self {
        Self {
            pack,
            pipelines: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn init(&mut self, wm: &WmRenderer) {
        let mut resources = HashMap::new();

        let block_atlas = wm.mc.texture_manager.atlases.load().get(BLOCK_ATLAS).unwrap().load();

        resources.insert("wm_texture_atlas_blocks".into(), CustomResource {
            update: None,
            data: Arc::new(ResourceInternal::Texture(TextureResource::Bindable(
                block_atlas.bindable_texture.clone()
            ), false)),
        });

        for (resource_id, definition) in &self.pack.resources.resources {
            let resource_id = resource_id.clone();

            match definition {
                ShorthandResourceConfig::Int(int) => {
                    let ssbo = UniformStorage::new(
                        wm,
                        bytemuck::cast_slice(&[*int]),
                        BufferUsages::STORAGE,
                        "ssbo",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::I64((*int, ssbo))),
                        },
                    );
                }
                ShorthandResourceConfig::Float(float) => {
                    let ssbo = UniformStorage::new(
                        wm,
                        bytemuck::cast_slice(&[*float]),
                        BufferUsages::UNIFORM,
                        "matrix",
                    );

                    resources.insert(
                        resource_id,
                        CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::F64((*float, ssbo))),
                        },
                    );
                }
                ShorthandResourceConfig::Mat3(mat3) => {
                    let ssbo = UniformStorage::new(
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
                            data: Arc::new(ResourceInternal::Mat3((
                                Mat3ValueOrMult::Value { value: *mat3 },
                                RwLock::new(matrix3),
                                ssbo,
                            ))),
                        },
                    );
                }
                ShorthandResourceConfig::Mat4(mat4) => {
                    let ssbo = UniformStorage::new(
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
                            data: Arc::new(ResourceInternal::Mat4((
                                Mat4ValueOrMult::Value { value: *mat4 },
                                RwLock::new(matrix4),
                                ssbo,
                            ))),
                        },
                    );
                }
                ShorthandResourceConfig::Longhand(longhand) => match &longhand.typed {
                    TypeResourceConfig::Texture3d { .. } => todo!(),
                    TypeResourceConfig::Texture2d {
                        src,
                        clear_after_frame,
                    } => {
                        if src.len() > 0 {
                            todo!()
                        } else {
                            let handle = wm.create_texture_handle(resource_id.clone(), TextureFormat::Bgra8Unorm);
                            resources.insert(resource_id, CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::Texture(TextureResource::Handle(handle), false)),
                            });
                        }
                    }
                    TypeResourceConfig::TextureDepth {
                        clear_after_frame,
                    } => {
                        let handle = wm.create_texture_handle(resource_id.clone(), TextureFormat::Depth32Float);
                        resources.insert(resource_id, CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::Texture(TextureResource::Handle(handle), true)),
                        });
                    }
                    TypeResourceConfig::F32 { value, .. } => {
                        let ssbo = UniformStorage::new(
                            wm,
                            bytemuck::cast_slice(&[*value]),
                            BufferUsages::STORAGE,
                            "ssbo",
                        );

                        resources.insert(
                            resource_id,
                            CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::F32((*value, ssbo))),
                            },
                        );
                    }
                    TypeResourceConfig::F64 { value, .. } => {
                        let ssbo = UniformStorage::new(
                            wm,
                            bytemuck::cast_slice(&[*value]),
                            BufferUsages::STORAGE,
                            "ssbo",
                        );

                        resources.insert(
                            resource_id,
                            CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::F64((*value, ssbo))),
                            },
                        );
                    }
                    TypeResourceConfig::I64 { value, .. } => {
                        let ssbo = UniformStorage::new(
                            wm,
                            bytemuck::cast_slice(&[*value]),
                            BufferUsages::STORAGE,
                            "ssbo",
                        );

                        resources.insert(
                            resource_id,
                            CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::I64((*value, ssbo))),
                            },
                        );
                    }
                    TypeResourceConfig::I32 { value, .. } => {
                        let ssbo = UniformStorage::new(
                            wm,
                            bytemuck::cast_slice(&[*value]),
                            BufferUsages::STORAGE,
                            "ssbo",
                        );

                        resources.insert(
                            resource_id,
                            CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::I32((*value, ssbo))),
                            },
                        );
                    }
                    TypeResourceConfig::Mat3(mat3) => {
                        let value = match mat3 {
                            Mat3ValueOrMult::Value { value } => *value,
                            Mat3ValueOrMult::Mult { .. } => [[0.0; 3]; 3],
                        };

                        let ssbo = UniformStorage::new(
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
                                data: Arc::new(ResourceInternal::Mat3((
                                    mat3.clone(),
                                    RwLock::new(value.into()),
                                    ssbo,
                                ))),
                            },
                        );
                    }
                    TypeResourceConfig::Mat4(mat4) => {
                        let value = match mat4 {
                            Mat4ValueOrMult::Value { value } => *value,
                            Mat4ValueOrMult::Mult { .. } => [[0.0; 4]; 4],
                        };

                        let ssbo = UniformStorage::new(
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
                                data: Arc::new(ResourceInternal::Mat4((
                                    mat4.clone(),
                                    RwLock::new(value.into()),
                                    ssbo,
                                ))),
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
                                    .map(|(index, uniform)| {
                                        match &*resources.get(&uniform.resource).expect(&uniform.resource).data {
                                            ResourceInternal::Texture(_, depth) => {
                                                layouts.get(if *depth { "texture_depth" } else { "texture" }).unwrap()
                                            }
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
                                    })
                                    .collect::<Vec<_>>(),
                                push_constant_ranges: if &definition.geometry == "wm_geo_terrain" {
                                    &[PushConstantRange {
                                        stages: wgpu::ShaderStages::VERTEX,
                                        range: 0..8,
                                    }]
                                } else {
                                    &[]
                                },
                            });

                    let pipeline =
                        wm.wgpu_state
                            .device
                            .create_render_pipeline(&RenderPipelineDescriptor {
                                label: None,
                                layout: Some(&pipeline_layout),
                                vertex: VertexState {
                                    module: &shader.shader,
                                    entry_point: "vert",
                                    buffers: &[match &definition.geometry[..] {
                                        "wm_geo_terrain" => Vertex::desc(),
                                        _ => unimplemented!("Unknown geometry"),
                                    }],
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
                                        .map(|target_texture| {
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

        self.resources = resources;
    }

    pub fn render(&self, wm: &WmRenderer, output_texture: &wgpu::TextureView) {
        let mut arena = WmArena::new(1024);

        let mut encoder = wm
            .wgpu_state
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        self.resources.iter().for_each(|(_, resource)| {
            match resource.update {
                None => {}
                Some(func) => func(resource, wm, &self.resources)
            }
        });

        //Reuse any RenderPasses wherever possible
        let mut last_config = None;

        let texture_handles = wm.texture_handles.read();

        //The first render pass that uses the framebuffer's depth buffer should clear it
        let mut should_clear_depth = true;
        self.pack.pipelines.pipelines.iter().for_each(|(name, config)| {
            if last_config != Some(config) {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &config.output.iter().map(|texture_name| {
                        let resource_definition = self.pack.resources.resources.get(texture_name);

                        //TODO: If the texture resource is defined as being cleared after each frame. Should use a HashMap to replace the should_clear_depth variable
                        let clear = match resource_definition {
                            Some(&ShorthandResourceConfig::Longhand(LonghandResourceConfig { typed: TypeResourceConfig::Texture2d { clear_after_frame: true, .. }, .. })) => true,
                            _ => false
                        };

                        Some(RenderPassColorAttachment {
                            view: match &texture_name[..] {
                                "wm_framebuffer_texture" => output_texture,
                                name => &arena.alloc(texture_handles.get(name).unwrap().bindable_texture.load()).tsv.view
                            },
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: true,
                            },
                        })
                    }).collect::<Vec<_>>(),
                    depth_stencil_attachment: config.depth.as_ref().map(|depth_texture| {
                        let will_clear_depth = should_clear_depth;
                        should_clear_depth = false;

                        RenderPassDepthStencilAttachment {
                            view: &arena.alloc(texture_handles.get(depth_texture).unwrap().bindable_texture.load()).tsv.view,
                            depth_ops: Some(Operations {
                                // load: if will_clear_depth { LoadOp::Clear(1.0) } else { LoadOp::Load },
                                load: LoadOp::Clear(1.0),
                                store: will_clear_depth,
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
                            for (pos, chunk_swap) in &*chunks {
                                let chunk = arena.alloc(chunk_swap.load());
                                let (chunk_vbo, verts) = arena.alloc(chunk.baked_layers.read()).get(layer.name()).unwrap();

                                render_pass.set_vertex_buffer(0, chunk_vbo.slice(..));

                                for (index, uniform) in &config.uniforms {
                                    let bind_group = match &*self.resources.get(&uniform.resource).unwrap().data {
                                        ResourceInternal::Texture(handle, _) => {
                                            match handle {
                                                TextureResource::Handle(handle) => &arena.alloc(handle.bindable_texture.load()).bind_group,
                                                TextureResource::Bindable(bindable) => &arena.alloc(bindable.load()).bind_group
                                            }
                                        }
                                        ResourceInternal::Blob(UniformStorage { bind_group, .. }) => {
                                            bind_group
                                        }
                                        ResourceInternal::Mat3((_, _, UniformStorage { bind_group, .. }))
                                        | ResourceInternal::Mat4((_, _, UniformStorage { bind_group, .. })) => {
                                            bind_group
                                        }
                                        ResourceInternal::F32((_, UniformStorage { bind_group, .. }))
                                        | ResourceInternal::F64((_, UniformStorage { bind_group, .. }))
                                        | ResourceInternal::U32((_, UniformStorage { bind_group, .. }))
                                        | ResourceInternal::I32((_, UniformStorage { bind_group, .. }))
                                        | ResourceInternal::I64((_, UniformStorage { bind_group, .. })) => bind_group
                                    };

                                    render_pass.set_bind_group(*index as u32, bind_group, &[]);
                                }

                                render_pass.set_push_constants(ShaderStages::VERTEX, 0, bytemuck::cast_slice(&chunk.pos));
                                render_pass.draw(0..verts.len() as u32, 0..1);
                            }
                        }
                    },
                    "wm_geo_entities" | "wm_geo_transparent" | "wm_geo_fluid" | "wm_geo_skybox" | "wm_geo_quad" => todo!("Specific geometry not yet implemented"),
                    _ => panic!("Unknown geometry {}", config.geometry)
                };
            }

            last_config = Some(config);
        });

        wm.wgpu_state.queue.submit([encoder.finish()]);
    }
}
