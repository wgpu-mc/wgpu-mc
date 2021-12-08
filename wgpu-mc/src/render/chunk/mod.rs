use crate::model::MeshVertex;
use crate::mc::chunk::{CHUNK_SECTIONS_PER, ChunkSection, CHUNK_AREA, CHUNK_HEIGHT, CHUNK_WIDTH, CHUNK_SECTION_HEIGHT, Chunk};
use parking_lot::RwLock;
use std::sync::Arc;
use crate::mc::block::{BlockState, Block};
use crate::WmRenderer;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use rayon::iter::{IntoParallelRefIterator, FromParallelIterator, IntoParallelIterator};
use crate::mc::block::model::{CubeOrComplexMesh, BlockstateVariantMesh};
use crate::mc::datapack::{NamespacedResource, BlockModel};
use crate::mc::BlockManager;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub lightmap_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl ChunkVertex {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
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

#[derive(Debug)]
pub struct BakedChunkPortionsContainer {
    pub top: BakedChunkPortion,
    pub bottom: BakedChunkPortion,
    pub north: BakedChunkPortion,
    pub east: BakedChunkPortion,
    pub south: BakedChunkPortion,
    pub west: BakedChunkPortion,
    pub nonstandard: BakedChunkPortion
}

fn get_block_mesh<'a>(block_manager: &'a BlockManager, state: &BlockState) -> Option<&'a BlockstateVariantMesh> {
    let block = block_manager.baked_block_variants.get_alt(&state.packed_key?)?;
    Some(block)
}

#[derive(Debug)]
pub struct BakedChunkPortion {
    pub buffer: wgpu::Buffer,
    //TODO: should this field be kept or is it unnecessary?
    pub vertices: Vec<ChunkVertex>
}

impl BakedChunkPortionsContainer {

    pub fn bake_portion(wm: &WmRenderer, chunk: &Chunk, section: &ChunkSection) -> Self {
        let block_manager = wm.mc.block_manager.read();
        let section_y = section.offset_y;

        //Generates the mesh for this chunk, hiding any full-block faces that aren't touching a transparent block
        let mut north_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 24);
        let mut east_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 24);
        let mut south_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 24);
        let mut west_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 24);
        let mut up_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 24);
        let mut down_vertices = Vec::with_capacity(CHUNK_AREA * CHUNK_SECTION_HEIGHT * 10);
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
                                v.position[1] + absolute_y as f32,
                                v.position[2] + z as f32 + chunk.pos.1 as f32
                            ],
                            tex_coords: v.tex_coords,
                            lightmap_coords: [
                                0.0,
                                0.0
                            ],
                            normal: v.normal
                        }
                    };

                    // println!("{:?}", block_state);

                    let baked_mesh = match get_block_mesh(&block_manager, &block_state) {
                        None => continue,
                        Some(mesh) => mesh,
                    };

                    match &baked_mesh.shape {
                        CubeOrComplexMesh::Cube(model) => {
                            let render_north = !(z > 0 && {
                                let north_block_mesh = get_block_mesh(
                                    &block_manager,
                                    &section.blocks[((z - 1) * CHUNK_WIDTH) + x]
                                );

                                match north_block_mesh {
                                    Some(block_mesh) => block_mesh.transparent_or_complex,
                                    None => false,
                                }
                            });

                            let render_south = !(z < 15 && {
                                let south_block_mesh = get_block_mesh(
                                    &block_manager,
                                    &section.blocks[((z + 1) * CHUNK_WIDTH) + x]
                                );

                                match south_block_mesh {
                                    Some(block_mesh) => block_mesh.transparent_or_complex,
                                    None => false,
                                }
                            });

                            let render_up = !(absolute_y < 255 && {
                                let up_block_mesh = get_block_mesh(
                                    &block_manager,
                                    &chunk.sections[(absolute_y + 1) / CHUNK_SECTION_HEIGHT].blocks[((z * CHUNK_WIDTH) + x) + ((y % CHUNK_SECTION_HEIGHT) * CHUNK_AREA)]
                                );

                                match up_block_mesh {
                                    Some(block_mesh) => block_mesh.transparent_or_complex,
                                    None => false,
                                }
                            });

                            let render_down = !(absolute_y > 0 && {
                                let down_block_mesh = get_block_mesh(
                                    &block_manager,
                                    &chunk.sections[(absolute_y - 1) / CHUNK_SECTION_HEIGHT].blocks[((z * CHUNK_WIDTH) + x) + ((y % CHUNK_SECTION_HEIGHT) * CHUNK_AREA)]
                                );

                                match down_block_mesh {
                                    Some(block_mesh) => block_mesh.transparent_or_complex,
                                    None => false,
                                }
                            });

                            let render_west = !(x > 0 && {
                                let west_block_mesh = get_block_mesh(
                                    &block_manager,
                                    &section.blocks[(z * CHUNK_WIDTH) + (x - 1)]
                                );

                                match west_block_mesh {
                                    Some(block_mesh) => block_mesh.transparent_or_complex,
                                    None => false,
                                }
                            });

                            let render_east = !(x < 15 && {
                                let east_block_mesh = get_block_mesh(
                                    &block_manager,
                                    &section.blocks[(z * CHUNK_WIDTH) + (x + 1)]
                                );

                                match east_block_mesh {
                                    Some(block_mesh) => block_mesh.transparent_or_complex,
                                    None => false,
                                }
                            });

                            if render_north {
                                match &model.north {
                                    None => {}
                                    Some(north) =>
                                        north_vertices.extend(north.iter().map(mapper))
                                };
                            }
                            if render_east {
                                match &model.east {
                                    None => {}
                                    Some(east) =>
                                        east_vertices.extend(east.iter().map(mapper))
                                };
                            }
                            if render_south {
                                match &model.south {
                                    None => {}
                                    Some(south) =>
                                        south_vertices.extend(south.iter().map(mapper))
                                };
                            }
                            if render_west {
                                match &model.north {
                                    None => {}
                                    Some(west) =>
                                        west_vertices.extend(west.iter().map(mapper))
                                };
                            }
                            if render_up {
                                match &model.up {
                                    None => {}
                                    Some(up) =>
                                        up_vertices.extend(up.iter().map(mapper))
                                };
                            }
                            if render_down {
                                match &model.north {
                                    None => {}
                                    Some(down) =>
                                        down_vertices.extend(down.iter().map(mapper))
                                };
                            }
                        }

                        CubeOrComplexMesh::Custom(model) => {
                            // let vertex_chain = model.iter().flat_map(|faces| {
                            //     [
                            //         faces.north.iter().map(mapper),
                            //         faces.east.iter().map(mapper),
                            //         faces.south.iter().map(mapper),
                            //         faces.west.iter().map(mapper),
                            //         faces.up.iter().map(mapper),
                            //         faces.down.iter().map(mapper)
                            //     ]
                            // }).flatten();
                            //
                            // other_vertices.extend(vertex_chain);

                            other_vertices.extend(
                                model.iter().flat_map(|faces| {
                                    [
                                        faces.north.as_ref(),
                                        faces.east.as_ref(),
                                        faces.south.as_ref(),
                                        faces.west.as_ref(),
                                        faces.up.as_ref(),
                                        faces.down.as_ref()
                                    ]
                                })
                                    .filter_map(|face| face)
                                    .flatten()
                                    .map(mapper)
                            );
                        }
                    }
                }
            }
        }

        let top_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&up_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let bottom_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&down_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let north_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&north_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let east_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&east_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let south_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&south_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let west_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&west_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        let nonstandard_buffer = wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&other_vertices[..]),
            usage: wgpu::BufferUsages::VERTEX
        });

        // println!("north verts: {}", north_vertices.len());

        Self {
            top: BakedChunkPortion { buffer: top_buffer, vertices: up_vertices },
            bottom: BakedChunkPortion { buffer: bottom_buffer, vertices: down_vertices },
            north: BakedChunkPortion { buffer: north_buffer, vertices: north_vertices },
            east: BakedChunkPortion { buffer: east_buffer, vertices: east_vertices },
            south: BakedChunkPortion { buffer: south_buffer, vertices: south_vertices },
            west: BakedChunkPortion { buffer: west_buffer, vertices: west_vertices },
            nonstandard: BakedChunkPortion { buffer: nonstandard_buffer, vertices: other_vertices }
        }
    }

}

#[derive(Debug)]
pub struct BakedChunk {
    pub sections: Arc<[BakedChunkPortionsContainer]>
}

impl BakedChunk {
    #[must_use]
    pub fn bake(wm: &WmRenderer, chunk: &Chunk) -> Self {
        Self {
            sections: chunk.sections.iter().map(|section| {
                BakedChunkPortionsContainer::bake_portion(wm, chunk, section)
            }).collect::<Arc<[BakedChunkPortionsContainer]>>()
        }
    }
}