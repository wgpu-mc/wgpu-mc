use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bytemuck::Pod;
use cgmath::{Matrix3, Matrix4, SquareMatrix};

use serde::Deserialize;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferUsages, Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, TextureFormat};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::render::pipeline::WmPipeline;
use crate::render::shaderpack::{LonghandResourceConfig, Mat3ValueOrMult, Mat4ValueOrMult, ShaderPackConfig, ShorthandResourceConfig, TypeResourceConfig};
use crate::texture::{BindableTexture, TextureHandle, TextureSamplerView};
use crate::util::{SSBO, WmArena};
use crate::WmRenderer;

fn mat3_update(resource: &CustomResource, wm: &WmRenderer, resources: &HashMap<String, CustomResource>) {
    let mut mat3 = Matrix4::<f32>::identity();

    if let Mat3ValueOrMult::Mult {
        mult
    } = &resource.data {
        mult.iter().for_each(|mat_name| {
            let resource = resources.get(mat_name).unwrap();

            match &*resource.data {
                ResourceInternal::Mat3((_, lock, _, _)) => {
                    mat3 = mat3 * lock.read();
                },
                _ => panic!("Invalid config. Mat3 resource multiplication should only ever point to other Mat3s")
            }
        })
    }
}

fn mat4_update(resource: &CustomResource, wm: &WmRenderer, resources: &HashMap<String, CustomResource>) {
    let mut mat4 = Matrix4::<f32>::identity();

    if let Mat4ValueOrMult::Mult {
        mult
    } = &resource.data {
        mult.iter().for_each(|mat_name| {
            let resource = resources.get(mat_name).unwrap();

            match &*resource.data {
                ResourceInternal::Mat4((_, lock, _, _)) => {
                    mat4 = mat4 * lock.read();
                },
                _ => panic!("Invalid config. Mat4 resource multiplication should only ever point to other Mat4s")
            }
        })
    }
}

enum ResourceInternal {
    Texture(TextureHandle),
    Blob(SSBO),
    Mat3((Mat3ValueOrMult, RwLock<Matrix3<f32>>, SSBO)),
    Mat4(((Mat4ValueOrMult, RwLock<Matrix4<f32>>, SSBO))),
    F32((f32, SSBO)),
    F64((f64, SSBO)),
    U32((u32, SSBO)),
    I32((i32, SSBO)),
    I64((i64, SSBO)),
}

struct CustomResource {
    //If this resource is updated each frame, this is what needs to be called
    update: Option<fn (&Self, &WmRenderer, &HashMap<String, CustomResource>)>,
    data: Arc<ResourceInternal>
}

pub struct ShaderGraph {
    pub pack: ShaderPackConfig,
    pub resources: RefCell<HashMap<String, CustomResource>>
}

impl ShaderGraph {
    fn init(&self, wm: &WmRenderer) {
        let mut resources = HashMap::new();

        for (resource_id, definition) in self.pack.resources.resources {
            match definition {
                ShorthandResourceConfig::Int(int) => {
                    let ssbo = SSBO::new(wm, bytemuck::cast_slice(&[int]), BufferUsages::STORAGE, false);

                    resources.insert(resource_id, CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::I64((int, ssbo))),
                    });
                }
                ShorthandResourceConfig::Float(float) => {
                    let ssbo = SSBO::new(wm, bytemuck::cast_slice(&[float]), BufferUsages::STORAGE, false);

                    resources.insert(resource_id, CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::F64((float, ssbo))),
                    });
                }
                ShorthandResourceConfig::Mat3(mat3) => {
                    let ssbo = SSBO::new(wm, bytemuck::cast_slice(&mat3), BufferUsages::STORAGE, false);

                    resources.insert(resource_id, CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::Mat3((Mat3ValueOrMult::Value { value: mat3 }, RwLock::new(mat3.into()), ssbo))),
                    });
                }
                ShorthandResourceConfig::Mat4(mat4) => {
                    let ssbo = SSBO::new(wm, bytemuck::cast_slice(&mat4), BufferUsages::STORAGE, false);

                    resources.insert(resource_id, CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::Mat4((Mat4ValueOrMult::Value { value: mat4 }, RwLock::new(mat4.into()), ssbo))),
                    });
                }
                ShorthandResourceConfig::Longhand(longhand) => {
                    match longhand.typed {
                        TypeResourceConfig::Texture3d { .. } => todo!(),
                        TypeResourceConfig::Texture2d { src, clear_after_frame } => {
                            wm.create_texture_handle(resource_id, TextureFormat::Bgra8Unorm);
                        },
                        TypeResourceConfig::TextureDepth { src, clear_after_frame } => {
                            wm.create_texture_handle(resource_id, TextureFormat::Depth32Float);
                        },
                        TypeResourceConfig::F32 { value, .. } => {
                            let ssbo = SSBO::new(wm, bytemuck::cast_slice(&[value]), BufferUsages::STORAGE, false);

                            resources.insert(resource_id, CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::F32((value, ssbo))),
                            });
                        }
                        TypeResourceConfig::F64 { value, .. } => {
                            let ssbo = SSBO::new(wm, bytemuck::cast_slice(&[value]), BufferUsages::STORAGE, false);

                            resources.insert(resource_id, CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::F64((value, ssbo))),
                            });
                        }
                        TypeResourceConfig::I64 { value, .. } => {
                            let ssbo = SSBO::new(wm, bytemuck::cast_slice(&[value]), BufferUsages::STORAGE, false);

                            resources.insert(resource_id, CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::I64((value, ssbo))),
                            });
                        }
                        TypeResourceConfig::I32 { value, .. } => {
                            let ssbo = SSBO::new(wm, bytemuck::cast_slice(&[value]), BufferUsages::STORAGE, false);

                            resources.insert(resource_id, CustomResource {
                                update: None,
                                data: Arc::new(ResourceInternal::I32((value, ssbo))),
                            });
                        }
                        TypeResourceConfig::Mat3(mat3) => {
                            let value = match mat3 {
                                Mat3ValueOrMult::Value { value } => value,
                                Mat3ValueOrMult::Mult { .. } => [[0.0; 3]; 3]
                            };

                            let ssbo = SSBO::new(wm, bytemuck::cast_slice(&value), BufferUsages::STORAGE, false);

                            resources.insert(resource_id, CustomResource {
                                update: match mat3 {
                                    Mat3ValueOrMult::Value { .. } => None,
                                    Mat3ValueOrMult::Mult { .. } => Some(mat3_update)
                                },
                                data: Arc::new(ResourceInternal::Mat3((mat3, RwLock::new(value.into()), ssbo))),
                            });
                        }
                        TypeResourceConfig::Mat4(mat4) => {
                            let value = match mat4 {
                                Mat4ValueOrMult::Value { value } => value,
                                Mat4ValueOrMult::Mult { .. } => [[0.0; 4]; 4]
                            };

                            let ssbo = SSBO::new(wm, bytemuck::cast_slice(&value), BufferUsages::STORAGE, false);

                            resources.insert(resource_id, CustomResource {
                                update: match mat4 {
                                    Mat4ValueOrMult::Value { .. } => None,
                                    Mat4ValueOrMult::Mult { .. } => Some(mat4_update)
                                },
                                data: Arc::new(ResourceInternal::Mat4((mat4, RwLock::new(value.into()), ssbo))),
                            });
                        }
                        TypeResourceConfig::Blob { .. } => todo!()
                    }
                }
            }
        }

        *self.resources.borrow_mut() = resources;
    }

    fn render(
        &self,
        wm: &WmRenderer,
    ) {
        let mut encoder = wm.wgpu_state.device.create_command_encoder(&CommandEncoderDescriptor {
            label: None,
        });

        let mut arena = WmArena::new(1024);
        let resources = self.resources.borrow();

        let mut last_config = None;
        let mut last_render_pass = None;

        let texture_handles = wm.texture_handles.read();

        self.pack.pipelines.pipelines.iter().for_each(|(name, config)| {
            if last_config != config {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &config.output.iter().map(|texture_name| {
                        let resource_definition = self.pack.resources.resources.get(texture_name);

                        //If the texture resource is defined as being cleared after each frame
                        let clear = match resource_definition {
                            Some(&ShorthandResourceConfig::Longhand(LonghandResourceConfig { typed: TypeResourceConfig::Texture2d { clear_after_frame: true, .. }, .. })) => true,
                            _ => false
                        };

                        Some(RenderPassColorAttachment {
                            view: &texture_handles.get(texture_name).unwrap().bindable_texture.load().tsv.view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                //Only write to the texture if its specified to be cleared
                                store: clear,
                            },
                        })
                    }).collect::<Vec<_>>(),
                    depth_stencil_attachment: None,
                });

                match &config.geometry[..] {
                    "wm_geo_terrain" => {
                        let layers = wm.pipelines.load().chunk_layers.load();
                        let chunks = wm.mc.chunks.loaded_chunks.read();

                        for layer in &**layers {
                            for (pos, chunk_swap) in &*chunks {
                                let chunk = chunk_swap.load();
                                let (chunk_vbo, verts) = chunk.baked_layers.read().get(layer.name()).unwrap();

                                render_pass.set_vertex_buffer(0, chunk_vbo.buffer.slice(..));

                                for (index, uniform) in config.uniforms {
                                    let bind_group = match &*resources.get(&uniform.resource).unwrap().data {
                                        ResourceInternal::Texture(handle) => {
                                            &arena.alloc(handle.bindable_texture.load()).bind_group
                                        }
                                        ResourceInternal::Blob(SSBO { bind_group, .. }) => {
                                            bind_group
                                        }
                                        ResourceInternal::Mat3((_, _, _, SSBO { bind_group, .. }))
                                        | ResourceInternal::Mat4((_, _, _, SSBO { bind_group, .. })) => {
                                            bind_group
                                        }
                                        ResourceInternal::F32((_, SSBO { bind_group, .. }))
                                        | ResourceInternal::F64((_, SSBO { bind_group, .. }))
                                        | ResourceInternal::U32((_, SSBO { bind_group, .. }))
                                        | ResourceInternal::I32((_, SSBO { bind_group, .. }))
                                        | ResourceInternal::I64((_, SSBO { bind_group, .. })) => bind_group
                                    };

                                    render_pass.set_bind_group(index as u32, bind_group, &[]);
                                }

                                render_pass.set_bind_group(0, &chunk.pos_buffer.bind_group, &[]);
                                render_pass.draw(0..verts.len() as u32, 0..1);
                            }
                        }
                    },
                    "wm_geo_entities" | "wm_geo_transparent" | "wm_geo_fluid" | "wm_geo_skybox" | "wm_geo_quad" => todo!("Specific geometry not yet implemented"),
                    _ => panic!("Unknown geometry {}", config.geometry)
                };

                last_render_pass = Some(render_pass);
            }

            last_config = Some(config);
        });
    }
}
