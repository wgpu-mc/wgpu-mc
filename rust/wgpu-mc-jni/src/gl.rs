use std::cmp::max;
use std::collections::HashMap;
use std::mem::align_of;
use std::ops::Range;
use std::sync::Arc;
use std::vec::Vec;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use wgpu_mc::render::graph::{
    set_push_constants, BoundPipeline, Geometry, RenderGraph, WmBindGroup,
};
use wgpu_mc::texture::BindableTexture;
use wgpu_mc::util::WmArena;
use wgpu_mc::wgpu::{vertex_attr_array, Buffer, IndexFormat};
use wgpu_mc::{wgpu, WmRenderer};

pub static GL_ALLOC: Lazy<RwLock<HashMap<u32, GlTexture>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
pub static GL_COMMANDS: Lazy<RwLock<(Vec<GLCommand>, Vec<GLCommand>)>> =
    Lazy::new(|| RwLock::new((Vec::new(), Vec::new())));

#[derive(Clone, Debug)]
pub enum GLCommand {
    SetMatrix(Mat4),
    SetColor([f32; 4]),
    #[allow(unused)]
    ClearColor([f32; 3]),
    UsePipeline(usize),
    SetVertexBuffer(Vec<u8>),
    SetIndexBuffer(Vec<u32>),
    DrawIndexed(u32),
    #[allow(unused)]
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

                vertex.pos[0..3].copy_from_slice(&vert[0..3]);
                vertex.pos[3] = 1.0;
                vertex.color[0..3].copy_from_slice(&vert[3..6]);
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

                vertex.pos[0..3].copy_from_slice(&vert[0..3]);
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

                vertex.pos[0..3].copy_from_slice(&vert[0..3]);
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

                vertex.pos[0..3].copy_from_slice(&vert[0..3]);
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
    color: [f32; 4],
    texture: Option<u32>,
}

#[derive(Debug)]
struct IndexedDraw {
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u32>,
    count: u32,
    matrix: [[f32; 4]; 4],
    color: [f32; 4],
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

        len..self.data.len() as u64
    }
}

#[derive(Debug)]
pub struct ElectrumGeometry {
    pub pool: Arc<Buffer>,
    pub last_bytes: Option<Vec<u8>>,
}

impl Geometry for ElectrumGeometry {
    fn render<'graph: 'pass + 'arena, 'pass, 'arena: 'pass>(
        &mut self,
        wm: &WmRenderer,
        _render_graph: &'graph RenderGraph,
        bound_pipeline: &'graph BoundPipeline,
        render_pass: &mut wgpu::RenderPass<'pass>,
        arena: &WmArena<'arena>,
    ) {
        let mut buffer_pool = BufferPool { data: Vec::new() };

        let (_, commands) = {
            GL_COMMANDS.read().clone() //Free the lock as soon as possible
        };

        let mut calls = vec![];

        let mut vertex_buffer = vec![];
        let mut index_buffer = vec![];
        let mut color = [1.0; 4];
        let mut matrix = Mat4::IDENTITY;
        let mut texture = None;
        let mut pipeline_state = None;

        let textures_read = GL_ALLOC.read();

        for command in commands {
            match command {
                GLCommand::SetColor(new_color) => {
                    color = new_color;
                }
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
                        matrix: Mat4::IDENTITY.to_cols_array_2d(),
                        color: [1.0; 4],
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
                        vertex_buffer: std::mem::take(&mut vertex_buffer),
                        index_buffer: std::mem::take(&mut index_buffer),
                        count,
                        matrix: matrix.to_cols_array_2d(),
                        texture: texture.take(),
                        color,
                        pipeline_state: pipeline_state.take().unwrap(),
                    }));
                }
                GLCommand::Draw(count) => {
                    calls.push(DrawCall::Verts(Draw {
                        vertex_buffer: std::mem::take(&mut vertex_buffer),
                        count,
                        matrix: matrix.to_cols_array_2d(),
                        color,
                        texture: texture.take(),
                    }));
                }
                GLCommand::AttachTexture(index, id) => {
                    assert_eq!(index, 0);
                    texture = Some(id as u32);
                }
            }
        }

        for call in calls {
            match call {
                DrawCall::Verts(draw) => {
                    let texture = match draw.texture {
                        None => continue,
                        Some(texture_id) => {
                            if let Some(gl_texture) = textures_read.get(&texture_id) {
                                gl_texture
                            } else {
                                continue;
                            }
                        }
                    };

                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Resource(name) => match &name[..] {
                                "@texture_electrum_gui" => {
                                    let bindable =
                                        texture.bindable_texture.as_ref().unwrap().clone();
                                    render_pass.set_bind_group(
                                        *index,
                                        &arena.alloc(bindable).bind_group,
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

                    render_pass.set_pipeline(&bound_pipeline.pipeline);

                    let mut push_constants = HashMap::new();
                    push_constants.insert(
                        "@pc_mat4_model".into(),
                        (
                            Vec::from(bytemuck::cast_slice(&draw.matrix)),
                            wgpu::ShaderStages::VERTEX,
                        ),
                    );
                    push_constants.insert(
                        "@pc_electrum_color".into(),
                        (
                            Vec::from(bytemuck::cast_slice(&draw.color)),
                            wgpu::ShaderStages::FRAGMENT,
                        ),
                    );
                    set_push_constants(&bound_pipeline.config, render_pass, Some(push_constants));

                    let buffer_slice = buffer_pool.allocate(&draw.vertex_buffer);

                    render_pass
                        .set_vertex_buffer(0, arena.alloc(self.pool.clone()).slice(buffer_slice));
                    render_pass.draw(0..draw.count, 0..1);
                }
                DrawCall::Indexed(draw) => {
                    let texture = match draw.texture {
                        None => continue,
                        Some(texture_id) => {
                            if let Some(gl_texture) = textures_read.get(&texture_id) {
                                gl_texture
                            } else {
                                continue;
                            }
                        }
                    };

                    for (index, bind_group) in bound_pipeline.bind_groups.iter() {
                        match bind_group {
                            WmBindGroup::Resource(name) => match &name[..] {
                                "@texture_electrum_gui" => {
                                    let bindable =
                                        texture.bindable_texture.as_ref().unwrap().clone();
                                    render_pass.set_bind_group(
                                        *index,
                                        &arena.alloc(bindable).bind_group,
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

                    render_pass.set_pipeline(&bound_pipeline.pipeline);

                    let mut push_constants = HashMap::new();
                    push_constants.insert(
                        "@pc_mat4_model".into(),
                        (
                            Vec::from(bytemuck::cast_slice(&draw.matrix)),
                            wgpu::ShaderStages::VERTEX,
                        ),
                    );
                    push_constants.insert(
                        "@pc_electrum_color".into(),
                        (
                            Vec::from(bytemuck::cast_slice(&draw.color)),
                            wgpu::ShaderStages::FRAGMENT,
                        ),
                    );
                    set_push_constants(&bound_pipeline.config, render_pass, Some(push_constants));

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

        match &self.last_bytes {
            None => {}
            Some(bytes) => {
                if bytes == &buffer_pool.data {
                    return;
                }
            }
        }

        wm.display
            .queue
            .write_buffer(&self.pool, 0, &buffer_pool.data);

        self.last_bytes = Some(buffer_pool.data);
    }
}
