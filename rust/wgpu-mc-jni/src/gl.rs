use std::cmp::max;
use std::vec::Vec;

use parking_lot::RwLock;

use wgpu_mc::texture::{BindableTexture, TextureHandle};

use std::sync::Arc;

use arc_swap::ArcSwap;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::mem::{align_of, replace};
use std::ops::Range;
use wgpu_mc::mc::chunk::ChunkPos;
use wgpu_mc::render::graph::{
    bind_uniforms, set_push_constants, CustomResource, GeometryCallback, ResourceInternal,
    ShaderGraph, TextureResource,
};
use wgpu_mc::render::shaderpack::{Mat4, Mat4ValueOrMult, PipelineConfig};
use wgpu_mc::util::{BindableBuffer, WmArena};
use wgpu_mc::wgpu::{
    vertex_attr_array, Buffer, BufferUsages, IndexFormat, RenderPass,
    SurfaceConfiguration,
};
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

#[derive(Debug, Pod, Zeroable, Copy, Clone)]
#[repr(C)]
pub struct ElectrumVertex {
    pub pos: [f32; 4],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub use_uv: u32,
}

impl ElectrumVertex {
    pub const VAO: [wgpu::VertexAttribute; 4] = vertex_attr_array![
        0 => Float32x4,
        1 => Float32x2,
        2 => Float32x4,
        3 => Uint32
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

                let color: u32 = bytemuck::cast(vert[5]);
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

    pub fn map_pos_color_uint(verts: &[[f32; 4]]) -> Vec<ElectrumVertex> {
        verts
            .iter()
            .map(|vert| {
                let mut vertex = ElectrumVertex::zeroed();

                (&mut vertex.pos[0..3]).copy_from_slice(&vert[0..3]);
                vertex.pos[3] = 1.0;

                let color: u32 = bytemuck::cast(vert[3]);
                let r = (color & 0xff) as f32 / 255.0;
                let g = ((color >> 8) & 0xff) as f32 / 255.0;
                let b = ((color >> 16) & 0xff) as f32 / 255.0;
                let a = ((color >> 24) & 0xff) as f32 / 255.0;

                vertex.color = [r, g, b, a];
                vertex.use_uv = 0;

                vertex
            })
            .collect()
    }

    pub fn map_pos_color_uv_light(verts: &[[u8; 28]]) -> Vec<ElectrumVertex> {
        verts
            .iter()
            .map(|vert| {
                let mut vertex = ElectrumVertex::zeroed();

                //Because of alignment issues we can't use bytemuck here
                vertex.pos[0] = f32::from_ne_bytes(vert[0..4].try_into().unwrap());
                vertex.pos[1] = f32::from_ne_bytes(vert[4..8].try_into().unwrap());
                vertex.pos[2] = f32::from_ne_bytes(vert[8..12].try_into().unwrap());
                vertex.pos[3] = 1.0;

                let color: u32 = u32::from_ne_bytes(vert[12..16].try_into().unwrap());
                let r = (color & 0xff) as f32 / 255.0;
                let g = ((color >> 8) & 0xff) as f32 / 255.0;
                let b = ((color >> 16) & 0xff) as f32 / 255.0;
                let a = ((color >> 24) & 0xff) as f32 / 255.0;

                vertex.color = [r, g, b, a];
                vertex.use_uv = 1;

                vertex.uv[0] = f32::from_ne_bytes(vert[16..20].try_into().unwrap());
                vertex.uv[1] = f32::from_ne_bytes(vert[20..24].try_into().unwrap());

                vertex
            })
            .collect()
    }
}

#[derive(Debug)]
struct Draw {
    vertex_buffer: Vec<u8>,
    count: u32,
    matrix: [[f32; 4]; 4],
    texture: Option<u32>,
    pipeline_state: PipelineState,
}

#[derive(Debug)]
struct IndexedDraw {
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u32>,
    count: u32,
    matrix: [[f32; 4]; 4],
    texture: Option<u32>,
    pipeline_state: PipelineState,
}

#[derive(Debug)]
enum DrawCall {
    Verts(Draw),
    Indexed(IndexedDraw),
}

#[derive(Debug)]
enum PipelineState {
    PositionColorUint,
    PositionUv,
    PositionColorF32,
    PositionUvColor,
    PositionColorUvLight,
}

fn augment_resources<'arena: 'resources, 'resources>(
    wm: &WmRenderer,
    resources: &'resources HashMap<String, CustomResource>,
    arena: &'arena WmArena<'arena>,
    texture: Arc<BindableTexture>,
    matrix: Mat4,
) -> &'arena HashMap<&'arena String, &'resources CustomResource> {
    arena.alloc(
        resources
            .into_iter()
            .chain([
                (
                    &*arena.alloc("wm_electrum_gl_texture".into()),
                    &*arena.alloc(CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::Texture(
                            TextureResource::Bindable(Arc::new(ArcSwap::new(texture))),
                            false,
                        )),
                    }),
                ),
                (
                    &*arena.alloc("wm_electrum_mat4".into()),
                    &*arena.alloc(CustomResource {
                        update: None,
                        data: Arc::new(ResourceInternal::Mat4(
                            Mat4ValueOrMult::Value { value: matrix },
                            Arc::new(RwLock::new(matrix.into())),
                            Arc::new(BindableBuffer::new(
                                wm,
                                bytemuck::cast_slice(&matrix),
                                BufferUsages::UNIFORM,
                                "matrix",
                            )),
                        )),
                    }),
                ),
            ])
            .collect(),
    )
}

pub struct BufferPool {
    pub data: Vec<u8>,
}

impl BufferPool {
    pub fn allocate<T: Copy + Pod + Zeroable>(&mut self, data: &[T]) -> Range<u64> {
        let len = self.data.len() as u64;

        let align = max(align_of::<T>(), 4);
        let pad = align - (len as usize % align);
        self.data.extend(vec![0u8; pad]);

        let len = self.data.len() as u64;
        self.data.extend(bytemuck::cast_slice(data));
        let range = len..self.data.len() as u64;
        range
    }
}

#[derive(Debug)]
pub struct ElectrumGeometry {
    pub blank: TextureHandle,
    pub pool: Arc<Buffer>,
}

impl GeometryCallback for ElectrumGeometry {
    fn render<'pass, 'resource: 'pass>(
        &self,
        wm: &WmRenderer,
        render_pass: &mut RenderPass<'pass>,
        graph: &'pass ShaderGraph,
        config: &PipelineConfig,
        resources: &'resource HashMap<String, CustomResource>,
        arena: &'resource WmArena<'resource>,
        surface_config: &SurfaceConfiguration,
        chunk_offset: ChunkPos,
    ) {
        let mut buffer_pool = BufferPool { data: Vec::new() };

        let (_, commands) = {
            GL_COMMANDS.read().clone() //Free the lock as soon as possible
        };

        let mut calls = vec![];

        let mut vertex_buffer = vec![];
        let mut index_buffer = vec![];
        let mut matrix = Matrix4::<f32>::identity();
        let mut texture = None;
        let mut pipeline_state = None;

        let textures_read = GL_ALLOC.read();

        for command in commands {
            match command {
                GLCommand::SetMatrix(new_matrix) => {
                    matrix = new_matrix;
                }
                GLCommand::ClearColor(color) => {
                    #[rustfmt::skip]
                    calls.push(DrawCall::Indexed(IndexedDraw {
                        vertex_buffer: Vec::from(
                            bytemuck::cast_slice(&[
                                -1.0, 1.0, 0.0, color[0], color[1], color[2],
                                1.0, 1.0, 0.0, color[0], color[1], color[2],
                                1.0, -1.0, 0.0, color[0], color[1], color[2],
                                -1.0, -1.0, 0.0, color[0], color[1], color[2]
                            ])
                        ),
                        index_buffer: vec![0,1,2,0,3,2],
                        count: 6,
                        matrix: Matrix4::<f32>::identity().into(),
                        texture: None,
                        pipeline_state: PipelineState::PositionColorF32,
                    }));
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
                        texture: texture.take(),
                        pipeline_state: pipeline_state.take().unwrap(),
                    }));
                }
                GLCommand::Draw(count) => {
                    calls.push(DrawCall::Verts(Draw {
                        vertex_buffer: replace(&mut vertex_buffer, vec![]),
                        count,
                        matrix: matrix.into(),
                        texture: texture.take(),
                        pipeline_state: pipeline_state.take().unwrap(),
                    }));
                }
                GLCommand::AttachTexture(index, id) => {
                    assert_eq!(index, 0);
                    texture = Some(id as u32);
                }
            }
        }

        //TODO: make gui rendering instanced to minimize draw calls by merging similar calls

        // let mut draws = HashMap::new();
        // let mut indexed_draws = HashMap::new();
        // let mut clears = HashMap::new();
        //
        // for call in calls {
        //     match call {
        //         DrawCall::Verts(draw) => {
        //             match draws.get_mut(&draw.texture) {
        //                 Some(assembled_draw) => {
        //                     assembled_draw.
        //                 },
        //                 None => {
        //                     draws.insert(draw.texture, draw);
        //                 }
        //             };
        //         }
        //         DrawCall::Indexed(_) => {}
        //         DrawCall::Clear(_) => {}
        //     }
        // }

        for call in calls {
            match call {
                DrawCall::Verts(draw) => {
                    let mut texture = self.blank.bindable_texture.load_full();

                    if let Some(texture_index) = draw.texture {
                        if let Some(gl_texture) = textures_read.get(&texture_index) {
                            texture = gl_texture.bindable_texture.as_ref().unwrap().clone();
                        }
                    }

                    let augmented_resources =
                        augment_resources(wm, &resources, arena, texture, draw.matrix);

                    bind_uniforms(config, augmented_resources, arena, render_pass);
                    set_push_constants(config, render_pass, None, surface_config, chunk_offset);

                    let buffer_slice = buffer_pool.allocate(&draw.vertex_buffer);

                    render_pass
                        .set_vertex_buffer(0, arena.alloc(self.pool.clone()).slice(buffer_slice));
                    render_pass.draw(0..draw.count, 0..1);
                }
                DrawCall::Indexed(draw) => {
                    let mut texture = self.blank.bindable_texture.load_full();

                    if let Some(texture_index) = draw.texture {
                        if let Some(gl_texture) = textures_read.get(&texture_index) {
                            texture = gl_texture.bindable_texture.as_ref().unwrap().clone();
                        }
                    }

                    let augmented_resources =
                        augment_resources(wm, &resources, arena, texture, draw.matrix);

                    bind_uniforms(config, augmented_resources, arena, render_pass);
                    set_push_constants(config, render_pass, None, surface_config, chunk_offset);

                    let vertices = match draw.pipeline_state {
                        PipelineState::PositionColorUint => ElectrumVertex::map_pos_color_uint(
                            bytemuck::cast_slice(&draw.vertex_buffer),
                        ),
                        PipelineState::PositionUv => {
                            ElectrumVertex::map_pos_uv(bytemuck::cast_slice(&draw.vertex_buffer))
                        }
                        PipelineState::PositionColorF32 => ElectrumVertex::map_pos_col_float3(
                            bytemuck::cast_slice(&draw.vertex_buffer),
                        ),
                        PipelineState::PositionUvColor => ElectrumVertex::map_pos_uv_color(
                            bytemuck::cast_slice(&draw.vertex_buffer),
                        ),
                        PipelineState::PositionColorUvLight => {
                            ElectrumVertex::map_pos_color_uv_light(
                                bytemuck::try_cast_slice(&draw.vertex_buffer).unwrap(),
                            )
                        }
                    };

                    let vert_slice = buffer_pool.allocate(&vertices);

                    let index_slice = buffer_pool.allocate(&draw.index_buffer);

                    let pool_alloc = arena.alloc(self.pool.clone());

                    render_pass.set_vertex_buffer(0, pool_alloc.slice(vert_slice));
                    render_pass
                        .set_index_buffer(pool_alloc.slice(index_slice), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..draw.count, 0, 0..1);
                }
            }
        }

        wm.wgpu_state
            .queue
            .write_buffer(&self.pool, 0, &buffer_pool.data);
    }
}
