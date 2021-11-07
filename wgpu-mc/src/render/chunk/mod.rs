use crate::model::MeshVertex;
use crate::mc::chunk::{CHUNK_SECTIONS_PER, ChunkSection, CHUNK_AREA, CHUNK_HEIGHT, CHUNK_WIDTH, CHUNK_SECTION_HEIGHT, Chunk};
use parking_lot::RwLock;
use std::sync::Arc;
use crate::mc::block::{BlockState, BlockShape};
use crate::WmRenderer;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use rayon::iter::{IntoParallelRefIterator, FromParallelIterator, IntoParallelIterator};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub lightmap_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl ChunkVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                //Texcoords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                //Lightmap
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct BakedChunkPortionsContainer {
    top: BakedChunkPortion,
    bottom: BakedChunkPortion,
    north: BakedChunkPortion,
    east: BakedChunkPortion,
    south: BakedChunkPortion,
    west: BakedChunkPortion,
    nonstandard: BakedChunkPortion
}

pub struct BakedChunkPortion {
    pub buffer: wgpu::Buffer,
    //TODO: should this field be kept or is it unnecessary?
    pub vertices: Vec<ChunkVertex>
}

impl BakedChunkPortionsContainer {

    pub fn bake_portion(wm: &WmRenderer, chunk: &Chunk, section: &ChunkSection) -> Self {
        let block_manager = wm.mc.block_manager.read();
        let section_y = section.offset_y;

        ///Generates the mesh for this chunk, hiding any full-block faces that aren't touching a transparent block
        let mut north_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 3);
        let mut east_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 3);
        let mut south_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 3);
        let mut west_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 3);
        let mut up_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 3);
        let mut bottom_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 3);
        let mut other_vertices = Vec::new();

        for y in 0..CHUNK_SECTION_HEIGHT {
            let absolute_y = y + section_y;

            for x in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let block_index = ((z * CHUNK_WIDTH) + x) + (y * CHUNK_AREA);
                    let block_state: BlockState = section.blocks[block_index];

                    let mapper = |v: &MeshVertex| {
                        ChunkVertex {
                            position: [
                                v.position[0] + x as f32 + chunk.pos.0 as f32,
                                v.position[1] + y as f32,
                                v.position[2] + z as f32 + chunk.pos.1 as f32
                            ],
                            tex_coords: v.tex_coords.clone(),
                            lightmap_coords: [
                                0.0,
                                0.0
                            ],
                            normal: v.normal.clone()
                        }
                    };

                    let block = block_manager.block_array.get(
                        match block_state.block {
                            Some(i) => i,
                            None => continue,
                        }).unwrap();

                    match block.get_shape() {
                        BlockShape::Cube(model) => {
                            let render_north = !(z > 0 && {
                                let north_block =
                                    &section.blocks[((z - 1) * CHUNK_WIDTH) + x];
                                match north_block.block {
                                    Some(_) => north_block.transparency,
                                    None => false,
                                }
                            });

                            let render_south = !(z < 15 && {
                                let south_block =
                                    &section.blocks[((z + 1) * CHUNK_WIDTH) + x];
                                match south_block.block {
                                    Some(_) => south_block.transparency,
                                    None => false,
                                }
                            });

                            let render_up = !(absolute_y < 255 && {
                                let up_block = &chunk.sections[(absolute_y + 1) / CHUNK_SECTION_HEIGHT].blocks[((z * CHUNK_WIDTH) + x) + ((y % CHUNK_SECTION_HEIGHT) * CHUNK_AREA)];
                                match up_block.block {
                                    Some(_) => up_block.transparency,
                                    None => false,
                                }
                            });

                            let render_down = !(absolute_y > 0 && {
                                let down_block = &chunk.sections[(absolute_y - 1) / CHUNK_SECTION_HEIGHT].blocks[((z * CHUNK_WIDTH) + x) + ((y % CHUNK_SECTION_HEIGHT) * CHUNK_AREA)];
                                match down_block.block {
                                    Some(_) => down_block.transparency,
                                    None => false,
                                }
                            });

                            let render_west = !(x > 0 && {
                                let west_block =
                                    &section.blocks[(z * CHUNK_WIDTH) + (x - 1)];
                                match west_block.block {
                                    Some(_) => west_block.transparency,
                                    None => false,
                                }
                            });

                            let render_east = !(x < 15 && {
                                let east_block =
                                    &section.blocks[(z * CHUNK_WIDTH) + (x + 1)];
                                match east_block.block {
                                    Some(_) => east_block.transparency,
                                    None => false,
                                }
                            });

                            if render_north {
                                north_vertices.extend(model.north.iter().map(mapper));
                            }
                            if render_east {
                                east_vertices.extend(model.east.iter().map(mapper));
                            }
                            if render_south {
                                south_vertices.extend(model.south.iter().map(mapper));
                            }
                            if render_west {
                                west_vertices.extend(model.west.iter().map(mapper));
                            }
                            if render_up {
                                up_vertices.extend(model.up.iter().map(mapper));
                            }
                            if render_down {
                                bottom_vertices.extend(model.down.iter().map(mapper));
                            }
                        }

                        BlockShape::Custom(model) => {
                            let vertex_chain = model.iter().flat_map(|faces| {
                                [
                                    faces.north.iter().map(mapper),
                                    faces.east.iter().map(mapper),
                                    faces.south.iter().map(mapper),
                                    faces.west.iter().map(mapper),
                                    faces.up.iter().map(mapper),
                                    faces.down.iter().map(mapper)
                                ]
                            }).flatten();

                            other_vertices.extend(vertex_chain);
                        }
                    }
                }
            }
        }

        let top_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&up_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let bottom_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&bottom_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let north_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&north_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let east_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&east_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let south_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&south_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let west_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&west_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let nonstandard_buffer = wm.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&other_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        Self {
            top: BakedChunkPortion { buffer: top_buffer, vertices: up_vertices },
            bottom: BakedChunkPortion { buffer: bottom_buffer, vertices: bottom_vertices },
            north: BakedChunkPortion { buffer: north_buffer, vertices: north_vertices },
            east: BakedChunkPortion { buffer: east_buffer, vertices: east_vertices },
            south: BakedChunkPortion { buffer: south_buffer, vertices: south_vertices },
            west: BakedChunkPortion { buffer: west_buffer, vertices: west_vertices },
            nonstandard: BakedChunkPortion { buffer: nonstandard_buffer, vertices: other_vertices }
        }
    }

}

pub struct BakedChunk {
    pub sections: Arc<[RwLock<BakedChunkPortionsContainer>]>
}

impl BakedChunk {
    pub fn bake(wm: &WmRenderer, chunk: &Chunk) -> Self {
        use rayon::iter::ParallelIterator;

        Self {
            sections: chunk.sections.iter().map(|section| {
                    RwLock::new(BakedChunkPortionsContainer::bake_portion(wm, chunk, section))
            }).collect::<Arc<[RwLock<BakedChunkPortionsContainer>]>>()
        }
    }
}