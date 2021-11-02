use crate::mc::block::{Block, BlockModel, BlockPos, BlockState};
use crate::model::ModelVertex;
use std::time::Instant;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use rayon::iter::IntoParallelRefMutIterator;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_SECTION_HEIGHT: usize = 1;
pub const CHUNK_SECTIONS_PER: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;

type ChunkPos = (i32, i32);

#[derive(Clone, Copy)]
pub struct ChunkSection {
    //16*16 area
    pub empty: bool,
    pub blocks: [BlockState; CHUNK_AREA * CHUNK_SECTION_HEIGHT],
}

type RawChunkSectionPaletted = [u8; 256];

struct RenderLayers {
    terrain: Box<[ChunkSection; CHUNK_SECTIONS_PER]>,
    transparent: Box<[ChunkSection; CHUNK_SECTIONS_PER]>,
    grass: Box<[ChunkSection; CHUNK_SECTIONS_PER]>
}

pub struct Chunk {
    pub pos: ChunkPos,
    pub sections: Box<[ChunkSection; CHUNK_SECTIONS_PER]>,
    pub vertices: Option<Vec<ModelVertex>>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_count: usize
}

impl Chunk {
    pub fn blockstate_at_pos(&self, pos: BlockPos) -> BlockState {
        let x = (pos.0 % 16) as usize;
        let y = (pos.1) as usize;
        let z = (pos.2 % 16) as usize;

        self.sections[y].blocks[(z * CHUNK_WIDTH) + x]
    }

    ///Generates the mesh for this chunk, hiding any full-block faces that aren't touching a transparent block
    pub fn generate_vertices(&mut self, blocks: &[Box<dyn Block>], pos_offset: ChunkPos) {
        let mut vertices = Vec::with_capacity(blocks.len() * 4 * 8);

        let sections = self.sections.as_ref();

        for y in 0..256 {
            let section_index = y / CHUNK_SECTION_HEIGHT;

            let section = sections[section_index];
            if section.empty {
                continue;
            }

            // let relative_section_y = y % CHUNK_SECTION_HEIGHT;
            for x in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let block_index = (z * CHUNK_WIDTH) + x;
                    let block_state: BlockState = section.blocks[block_index];

                    let mapper = |v: &ModelVertex| {
                        let mut vertex = *v;
                        vertex.position[0] += x as f32 + pos_offset.0 as f32;
                        vertex.position[1] += y as f32;
                        vertex.position[2] += z as f32 + pos_offset.1 as f32;

                        vertex
                    };

                    let block = blocks
                        .get(match block_state.block {
                            Some(i) => i,
                            None => continue,
                        })
                        .unwrap();

                    match block.get_model() {
                        BlockModel::Cube(model) => {
                            let render_north = !(z > 0 && {
                                let north_block =
                                    self.sections[y].blocks[((z - 1) * CHUNK_WIDTH) + x];
                                match north_block.block {
                                    Some(_) => north_block.transparency,
                                    None => false,
                                }
                            });

                            let render_south = !(z < 15 && {
                                let south_block =
                                    self.sections[y].blocks[((z + 1) * CHUNK_WIDTH) + x];
                                match south_block.block {
                                    Some(_) => south_block.transparency,
                                    None => false,
                                }
                            });

                            let render_up = !(y < 255 && {
                                let up_block = self.sections[y + 1].blocks[(z * CHUNK_WIDTH) + x];
                                match up_block.block {
                                    Some(_) => up_block.transparency,
                                    None => false,
                                }
                            });

                            let render_down = !(y > 0 && {
                                let down_block = self.sections[y - 1].blocks[(z * CHUNK_WIDTH) + x];
                                match down_block.block {
                                    Some(_) => down_block.transparency,
                                    None => false,
                                }
                            });

                            let render_west = !(x > 0 && {
                                let west_block =
                                    self.sections[y].blocks[(z * CHUNK_WIDTH) + (x - 1)];
                                match west_block.block {
                                    Some(_) => west_block.transparency,
                                    None => false,
                                }
                            });

                            let render_east = !(x < 15 && {
                                let east_block =
                                    self.sections[y].blocks[(z * CHUNK_WIDTH) + (x + 1)];
                                match east_block.block {
                                    Some(_) => east_block.transparency,
                                    None => false,
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
            usage: wgpu::BufferUsages::VERTEX
        }));
    }
}

pub struct ChunkManager {
    //Due to floating point inaccuracy at large distances,
    //we need to keep the model coordinates as close to 0,0,0 as possible
    pub chunk_origin: ChunkPos,
    pub loaded_chunks: Vec<Chunk>,
}

impl ChunkManager {
    pub fn new() -> Self {
        ChunkManager {
            chunk_origin: (0, 0),
            loaded_chunks: vec![],
        }
    }

    //TODO: parallelize
    // pub fn bake_meshes(&mut self, blocks: &[Box<dyn Block>]) {
    //     self.loaded_chunks.iter_mut().for_each(
    //         |chunk| chunk.generate_vertices(blocks, self.chunk_origin));
    // }
    //
    // pub fn upload_buffers(&mut self, device: &wgpu::Device) {
    //     self.loaded_chunks.iter_mut().for_each(|chunk| chunk.upload_buffer(device));
    // }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}
