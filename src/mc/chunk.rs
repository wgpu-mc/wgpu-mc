use std::collections::HashMap;
use crate::mc::block::{Block, BlockState, BlockPos, BlockDirection, BlockEntity, BlockModel};
use crate::mc::entity::Entity;
use crate::{InstanceRaw, Instance};
use cgmath::Quaternion;
use std::cell::RefCell;
use crate::model::ModelVertex;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use std::ops::Deref;

const CHUNK_WIDTH: usize = 16;
const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
const CHUNK_HEIGHT: usize = 256;

type ChunkPos = (u32, u32);

#[derive(Clone, Copy)]
pub struct ChunkSection { //16*16 area
    pub empty: bool,
    pub blocks: [
        BlockState; CHUNK_AREA
    ]
}

pub struct Chunk {
    pub pos: ChunkPos,
    pub sections: Box<[ChunkSection; CHUNK_HEIGHT]>,
    pub vertices: Option<Vec<ModelVertex>>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_count: usize
}

impl Chunk {
    pub fn blockstate_at_pos(&self, pos: BlockPos) -> BlockState {
        let x = (pos.0 % 16) as usize;
        let y = (pos.1) as usize;
        let z = (pos.2 % 16) as usize;

        self.sections[y].blocks[
            (z * CHUNK_WIDTH) + x
        ]
    }

    pub fn generate_vertices(&mut self, blocks: &Vec<Box<dyn Block>>) {
        let mut vertices = Vec::new();

        for y in 0..CHUNK_HEIGHT {
            let section = &self.sections[y];

            if section.empty {
                continue;
            }

            for x in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let block_state: BlockState = self.sections.deref()[y].blocks[(z * CHUNK_WIDTH) + x];
                    let block = blocks.get(block_state.block).unwrap();

                    let mapper = |v: &ModelVertex| {
                        let mut vertex = *v;
                        vertex.position[0] += (x as f32);
                        vertex.position[1] += (y as f32);
                        vertex.position[2] += (z as f32);

                        vertex
                    };

                    let north_neighbour = {

                    };

                    match block.get_model() {
                        BlockModel::Cube(model) => {
                            vertices.extend_from_slice(&model.north.iter().map(mapper).collect::<Vec<ModelVertex>>());
                            vertices.extend_from_slice(&model.east.iter().map(mapper).collect::<Vec<ModelVertex>>());
                            vertices.extend_from_slice(&model.south.iter().map(mapper).collect::<Vec<ModelVertex>>());
                            vertices.extend_from_slice(&model.west.iter().map(mapper).collect::<Vec<ModelVertex>>());
                            vertices.extend_from_slice(&model.up.iter().map(mapper).collect::<Vec<ModelVertex>>());
                            // vertices.extend_from_slice(&model.down.ite r().map(mapper).collect::<Vec<ModelVertex>>());
                        }
                        BlockModel::Custom(model) => {
                            model.iter().for_each(|faces| {
                                vertices.extend_from_slice(&faces.north.iter().map(mapper).collect::<Vec<ModelVertex>>());
                                vertices.extend_from_slice(&faces.east.iter().map(mapper).collect::<Vec<ModelVertex>>());
                                vertices.extend_from_slice(&faces.south.iter().map(mapper).collect::<Vec<ModelVertex>>());
                                vertices.extend_from_slice(&faces.west.iter().map(mapper).collect::<Vec<ModelVertex>>());
                                vertices.extend_from_slice(&faces.up.iter().map(mapper).collect::<Vec<ModelVertex>>());
                                vertices.extend_from_slice(&faces.down.iter().map(mapper).collect::<Vec<ModelVertex>>());
                            });
                        }
                    }
                }
            }
        }

        self.vertex_count = vertices.len();
        self.vertices = Option::Some(vertices);
    }

    pub fn upload_buffer(&mut self, device: &wgpu::Device) {
        self.vertex_buffer = Option::Some(
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(match &self.vertices {
                    None => panic!("Cannot upload chunk buffer, vertices have not been generated!"),
                    Some(v) => &v[..]
                }),
                usage: wgpu::BufferUsage::VERTEX
            })
        );
    }
}

pub struct ChunkManager {
    pub loaded_chunks: Vec<Chunk>
}

impl ChunkManager {
    pub fn new() -> Self {
        ChunkManager { loaded_chunks: vec![] }
    }
}