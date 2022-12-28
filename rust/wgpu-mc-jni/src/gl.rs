use std::vec::Vec;

use parking_lot::RwLock;

use wgpu_mc::texture::BindableTexture;

use std::sync::Arc;

use arc_swap::ArcSwap;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};
use once_cell::sync::{Lazy, OnceCell};
use std::collections::HashMap;
use std::mem::replace;
use wgpu_mc::render::graph::{
    bind_uniforms, set_push_constants, CustomResource, ResourceInternal, ShaderGraph,
    TextureResource,
};
use wgpu_mc::render::shaderpack;
use wgpu_mc::render::shaderpack::PipelineConfig;
use wgpu_mc::util::{UniformStorage, WmArena};
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_mc::wgpu::{vertex_attr_array, BufferUsages, IndexFormat, RenderPass, ShaderStages};
use wgpu_mc::{wgpu, WmRenderer};

pub static GL_ALLOC: Lazy<RwLock<HashMap<u32, GlTexture>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
pub static GL_COMMANDS: Lazy<RwLock<(Vec<GLCommand>, Vec<GLCommand>)>> =
    Lazy::new(|| RwLock::new((Vec::new(), Vec::new())));

#[derive(Clone, Debug)]
pub enum GLCommand {
    SetMatrix(Matrix4<f32>),
    ClearColor([f32; 3]),
    UsePipeline(usize),
    SetVertexBuffer(Vec<u8>),
    SetIndexBuffer(Vec<u32>),
    DrawIndexed(u32),
    Draw(u32),
    AttachTexture(u32, i32),
}

#[derive(Debug)]
pub struct GlTexture {
    pub width: u16,
    pub height: u16,
    pub bindable_texture: Option<Arc<BindableTexture>>,
    pub pixels: Vec<u8>,
}

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
pub struct ElectrumVertex {
    pub pos: [f32; 4],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub use_uv: u32,
}

impl ElectrumVertex {
    pub const VAO: [wgpu::VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x4,
        1 => Float32x2,
        2 => Float32x4
    ];
}

impl ElectrumVertex {
    pub fn map_pos_col_float3(verts: &[[f32; 6]]) -> Vec<ElectrumVertex> {
        verts
            .iter()
            .map(|vert| {
                let mut vertex = ElectrumVertex::zeroed();

                (&mut vertex.pos[0..3]).copy_from_slice(&vert[0..3]);
                vertex.pos[3] = 1.0;
                (&mut vertex.color[0..3]).copy_from_slice(&vert[3..6]);
                vertex.color[3] = 1.0;

                vertex
            })
            .collect()
    }

    pub fn map_pos_uv(verts: &[[f32; 5]]) -> Vec<ElectrumVertex> {
        verts
            .iter()
            .map(|vert| {
                let mut vertex = ElectrumVertex::zeroed();

                (&mut vertex.pos[0..3]).copy_from_slice(&vert[0..3]);
                vertex.pos[3] = 1.0;
                vertex.uv.copy_from_slice(&vert[3..5]);
                vertex.color = [1.0; 4];
                vertex.use_uv = 1;

                vertex
            })
            .collect()
    }

    pub fn map_pos_uv_color(verts: &[[f32; 6]]) -> Vec<ElectrumVertex> {
        verts
            .iter()
            .map(|vert| {
                let mut vertex = ElectrumVertex::zeroed();

                (&mut vertex.pos[0..3]).copy_from_slice(&vert[0..3]);
                vertex.pos[3] = 1.0;
                vertex.uv.copy_from_slice(&vert[3..5]);

                let color: u32 = bytemuck::cast(verts[5]);
                let r = (color & 0xff) as f32 / 255.0;
                let g = ((color >> 8) & 0xff) as f32 / 255.0;
                let b = ((color >> 16) & 0xff) as f32 / 255.0;
                let a = ((color >> 24) & 0xff) as f32 / 255.0;

                vertex.color = [r, g, b, a];
                vertex.use_uv = 1;

                vertex
            })
            .collect()
    }
}

#[derive(Debug)]
struct Draw<'a> {
    vertex_buffer: Vec<u8>,
    count: u32,
    matrix: [[f32; 4]; 4],
    textures: HashMap<u32, &'a GlTexture>,
    pipeline_state: PipelineState,
}

#[derive(Debug)]
struct IndexedDraw<'a> {
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u32>,
    count: u32,
    matrix: [[f32; 4]; 4],
    textures: HashMap<u32, &'a GlTexture>,
    pipeline_state: PipelineState,
}

#[derive(Debug)]
enum DrawCall<'a> {
    Verts(Draw<'a>),
    Indexed(IndexedDraw<'a>),
    Clear([f32; 3]),
}

#[derive(Debug)]
enum PipelineState {
    PositionColorUint,
    PositionUv,
    PositionColorF32,
    PositionUvColor,
    PositionColorUvLight,
}

pub fn electrum_gui_callback<'arena: 'pass, 'pass>(
    wm: &WmRenderer,
    render_pass: &mut RenderPass<'pass>,
    graph: &ShaderGraph,
    pipeline: &PipelineConfig,
    resources: &HashMap<&String, &'pass CustomResource>,
    arena: &'arena WmArena,
) {
    let (_, commands) = {
        GL_COMMANDS.read().clone() //Free the lock as soon as possible
    };

    let config = { wm.wgpu_state.surface.read().1.clone() };

    let mut calls = vec![];

    let mut vertex_buffer = vec![];
    let mut index_buffer = vec![];
    let mut matrix = Matrix4::<f32>::identity();
    let mut textures = HashMap::new();
    let mut pipeline_state = None;

    let textures_read = GL_ALLOC.read();

    for command in commands {
        match command {
            GLCommand::SetMatrix(new_matrix) => {
                matrix = new_matrix;
            }
            GLCommand::ClearColor(color) => {
                calls.push(DrawCall::Clear(color));
            }
            GLCommand::UsePipeline(pipeline) => {
                pipeline_state = Some(match pipeline {
                    0 => PipelineState::PositionColorUint,
                    1 => PipelineState::PositionUv,
                    2 => PipelineState::PositionColorF32,
                    3 => PipelineState::PositionColorUvLight,
                    4 => PipelineState::PositionUvColor,
                    _ => unimplemented!(),
                });
            }
            GLCommand::SetVertexBuffer(buffer) => {
                vertex_buffer = buffer;
            }
            GLCommand::SetIndexBuffer(buffer) => {
                index_buffer = buffer;
            }
            GLCommand::DrawIndexed(count) => {
                calls.push(DrawCall::Indexed(IndexedDraw {
                    vertex_buffer: replace(&mut vertex_buffer, vec![]),
                    index_buffer: replace(&mut index_buffer, vec![]),
                    count,
                    matrix: matrix.into(),
                    textures: replace(&mut textures, HashMap::new()),
                    pipeline_state: pipeline_state.take().unwrap(),
                }));
            }
            GLCommand::Draw(count) => {
                calls.push(DrawCall::Verts(Draw {
                    vertex_buffer: replace(&mut vertex_buffer, vec![]),
                    count,
                    matrix: matrix.into(),
                    textures: replace(&mut textures, HashMap::new()),
                    pipeline_state: pipeline_state.take().unwrap(),
                }));
            }
            GLCommand::AttachTexture(index, texture) => {
                textures.insert(index, textures_read.get(&index).unwrap());
            }
        }
    }

    println!("{calls:?}");

    let key = "wm_electrum_gl_texture".into();

    for call in calls {
        match call {
            DrawCall::Verts(draw) => {
                assert_eq!(draw.textures.len(), 1);

                let texture = draw.textures.get(&0).unwrap();
                let bindable = texture.bindable_texture.as_ref().unwrap().clone();

                let augmented_resources = resources
                    .clone()
                    .into_iter()
                    .chain([(
                        &key,
                        &*arena.alloc(CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::Texture(
                                TextureResource::Bindable(Arc::new(ArcSwap::new(bindable))),
                                false,
                            )),
                        }),
                    )])
                    .collect();

                bind_uniforms(pipeline, &augmented_resources, arena, render_pass);
                set_push_constants(pipeline, render_pass, None, &config);

                let buffer = wm
                    .wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: &draw.vertex_buffer,
                        usage: BufferUsages::VERTEX,
                    });

                render_pass.set_vertex_buffer(0, arena.alloc(buffer).slice(..));
                render_pass.draw(0..draw.count, 0..1);
            }
            DrawCall::Indexed(draw) => {
                assert_eq!(draw.textures.len(), 1);

                let texture = draw.textures.get(&0).unwrap();
                let bindable = texture.bindable_texture.as_ref().unwrap().clone();

                let augmented_resources = resources
                    .clone()
                    .into_iter()
                    .chain([(
                        &key,
                        &*arena.alloc(CustomResource {
                            update: None,
                            data: Arc::new(ResourceInternal::Texture(
                                TextureResource::Bindable(Arc::new(ArcSwap::new(bindable))),
                                false,
                            )),
                        }),
                    )])
                    .collect();

                bind_uniforms(pipeline, &augmented_resources, arena, render_pass);
                set_push_constants(pipeline, render_pass, None, &config);

                let vertices = match draw.pipeline_state {
                    PipelineState::PositionColorUint => continue,
                    PipelineState::PositionUv => {
                        ElectrumVertex::map_pos_uv(bytemuck::cast_slice(&draw.vertex_buffer))
                    }
                    PipelineState::PositionColorF32 => ElectrumVertex::map_pos_col_float3(
                        bytemuck::cast_slice(&draw.vertex_buffer),
                    ),
                    PipelineState::PositionUvColor => {
                        ElectrumVertex::map_pos_uv_color(bytemuck::cast_slice(&draw.vertex_buffer))
                    }
                    PipelineState::PositionColorUvLight => continue,
                };

                let vertex_buffer =
                    wm.wgpu_state
                        .device
                        .create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&vertices),
                            usage: BufferUsages::VERTEX,
                        });

                let index_buffer = wm
                    .wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&draw.index_buffer),
                        usage: BufferUsages::VERTEX,
                    });

                render_pass.set_vertex_buffer(0, arena.alloc(vertex_buffer).slice(..));
                render_pass
                    .set_index_buffer(arena.alloc(index_buffer).slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..draw.count, 0, 0..1);
            }
            DrawCall::Clear(color) => {
                render_pass.set_push_constants(
                    ShaderStages::FRAGMENT,
                    0,
                    bytemuck::cast_slice(&color),
                );
                render_pass.draw(0..6, 0..1);
            }
        }
    }
}
