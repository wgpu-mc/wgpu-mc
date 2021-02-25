use crate::mc::block::{Block, BlockModel, BlockPos, BlockState};
use crate::model::ModelVertex;
use std::time::Instant;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 256;

type ChunkPos = (u32, u32);

#[derive(Clone, Copy)]
pub struct ChunkSection {
    //16*16 area
    pub empty: bool,
    pub blocks: [BlockState; CHUNK_AREA],
}

pub struct Chunk {
    pub pos: ChunkPos,
    pub sections: Box<[ChunkSection; CHUNK_HEIGHT]>,
    pub vertices: Option<Vec<ModelVertex>>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_count: usize,
}

impl Chunk {
    pub fn blockstate_at_pos(&self, pos: BlockPos) -> BlockState {
        let x = (pos.0 % 16) as usize;
        let y = (pos.1) as usize;
        let z = (pos.2 % 16) as usize;

        self.sections[y].blocks[(z * CHUNK_WIDTH) + x]
    }

    pub fn generate_vertices(&mut self, blocks: &[Box<dyn Block>]) {
        let mut vertices = Vec::new();

        #[allow(unused_variables)] // TODO
        let instant = Instant::now();

        let sections = self.sections.as_ref();

        for (y, section) in sections.iter().enumerate().take(CHUNK_HEIGHT) {
            if section.empty {
                continue;
            }

            for x in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let block_state: BlockState = sections[y].blocks[(z * CHUNK_WIDTH) + x];

                    let block = blocks
                        .get(match block_state.block {
                            Some(i) => i,
                            None => continue,
                        })
                        .unwrap();

                    let mapper = |v: &ModelVertex| {
                        let mut vertex = *v;
                        vertex.position[0] += x as f32;
                        vertex.position[1] += y as f32;
                        vertex.position[2] += z as f32;

                        vertex
                    };

                    match block.get_model() {
                        BlockModel::Cube(model) => {
                            let render_north = !(z > 0 && {
                                let north_block =
                                    self.sections[y].blocks[((z - 1) * CHUNK_WIDTH) + x];
                                match north_block.block {
                                    Some(_) => !north_block.is_cube,
                                    None => true,
                                }
                            });

                            let render_south = !(z < 15 && {
                                let south_block =
                                    self.sections[y].blocks[((z + 1) * CHUNK_WIDTH) + x];
                                match south_block.block {
                                    Some(_) => !south_block.is_cube,
                                    None => true,
                                }
                            });

                            let render_up = !(y < 255 && {
                                let up_block = self.sections[y + 1].blocks[(z * CHUNK_WIDTH) + x];
                                match up_block.block {
                                    Some(_) => !up_block.is_cube,
                                    None => true,
                                }
                            });

                            let render_down = !(y > 0 && {
                                let down_block = self.sections[y - 1].blocks[(z * CHUNK_WIDTH) + x];
                                match down_block.block {
                                    Some(_) => !down_block.is_cube,
                                    None => true,
                                }
                            });

                            let render_west = !(x > 0 && {
                                let west_block =
                                    self.sections[y].blocks[(z * CHUNK_WIDTH) + (x - 1)];
                                match west_block.block {
                                    Some(_) => !west_block.is_cube,
                                    None => true,
                                }
                            });

                            let render_east = !(x < 15 && {
                                let east_block =
                                    self.sections[y].blocks[(z * CHUNK_WIDTH) + (x + 1)];
                                match east_block.block {
                                    Some(_) => !east_block.is_cube,
                                    None => true,
                                }
                            });

                            if render_north {
                                vertices.extend(model.north.iter().map(mapper));
                            }
                            if render_east {
                                vertices.extend(model.east.iter().map(mapper));
                            }
                            if render_south {
                                vertices.extend(model.south.iter().map(mapper));
                            }
                            if render_west {
                                vertices.extend(model.west.iter().map(mapper));
                            }
                            if render_up {
                                vertices.extend(model.up.iter().map(mapper));
                            }
                            if render_down {
                                vertices.extend(model.down.iter().map(mapper));
                            }
                        }

                        BlockModel::Custom(model) => {
                            for faces in model.iter() {
                                vertices.extend(faces.north.iter().map(mapper));
                                vertices.extend(faces.east.iter().map(mapper));
                                vertices.extend(faces.south.iter().map(mapper));
                                vertices.extend(faces.west.iter().map(mapper));
                                vertices.extend(faces.up.iter().map(mapper));
                                vertices.extend(faces.down.iter().map(mapper));
                            }
                        }
                    }
                }
            }
        }

        self.vertex_count = vertices.len();
        self.vertices = Some(vertices);
    }

    pub fn upload_buffer(&mut self, device: &wgpu::Device) {
        self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(match &self.vertices {
                Some(v) => &v,
                None => panic!("Cannot upload chunk buffer, vertices have not been generated!"),
            }),
            usage: wgpu::BufferUsage::VERTEX,
        }));
    }
}

pub struct ChunkManager {
    pub loaded_chunks: Vec<Chunk>,
}

impl ChunkManager {
    pub fn new() -> Self {
        ChunkManager {
            loaded_chunks: vec![],
        }
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}
