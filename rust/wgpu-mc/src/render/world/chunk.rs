use std::collections::HashMap;
use std::sync::Arc;
use crate::mc::chunk::{
    BlockStateProvider, Chunk, WorldBuffers, CHUNK_AREA, CHUNK_SECTION_HEIGHT, CHUNK_VOLUME,
    CHUNK_WIDTH,
};

use crate::mc::block::{ChunkBlockState, CubeOrComplexMesh, BlockstateKey, BlockMeshVertex, ModelMesh};
use bytemuck::Pod;

use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::mc::{BlockManager};
use crate::WmRenderer;

fn get_block<'a>(
    block_manager: &'a BlockManager,
    state: ChunkBlockState,
) -> Option<Arc<ModelMesh>> {
    let key = match state {
        ChunkBlockState::Air => return None,
        ChunkBlockState::State(key ) => key,
    };

    Some(block_manager.blocks.get_index(key.block as usize)?
        .1.get_model(key.augment))
}

#[derive(Debug)]
pub struct BakedChunkLayer<T: Copy + Pod> {
    pub top: Vec<T>,
    pub bottom: Vec<T>,
    pub north: Vec<T>,
    pub east: Vec<T>,
    pub south: Vec<T>,
    pub west: Vec<T>,
    pub nonstandard: Vec<T>,
}

impl<T: Copy + Pod> BakedChunkLayer<T> {
    pub fn new() -> Self {
        Self {
            top: vec![],
            bottom: vec![],
            north: vec![],
            east: vec![],
            south: vec![],
            west: vec![],
            nonstandard: vec![],
        }
    }

    pub fn upload(&self, wm: &WmRenderer) -> WorldBuffers {
        WorldBuffers {
            top: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.top[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.top.len(),
            ),
            bottom: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.bottom[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.bottom.len(),
            ),
            north: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.north[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.north.len(),
            ),
            south: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.south[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.south.len(),
            ),
            west: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.west[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.west.len(),
            ),
            east: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.east[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.east.len(),
            ),
            other: (
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.nonstandard[..]),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.nonstandard.len(),
            ),
        }
    }

    pub fn extend(&mut self, other: &Self) {
        self.top.extend(other.top.iter());
        self.bottom.extend(other.bottom.iter());
        self.north.extend(other.north.iter());
        self.south.extend(other.south.iter());
        self.west.extend(other.west.iter());
        self.east.extend(other.east.iter());
        self.nonstandard.extend(other.nonstandard.iter());
    }

    #[must_use]
    pub fn bake<Provider: BlockStateProvider>(
        block_manager: &BlockManager,
        chunk: &Chunk,
        mapper: fn(&BlockMeshVertex, f32, f32, f32) -> T,
        filter: Box<dyn Fn(BlockstateKey) -> bool>,
        state_provider: &Provider,
    ) -> Self {
        let chunk_world_x = (chunk.pos.0 * (CHUNK_WIDTH as i32));
        let chunk_world_z = (chunk.pos.1 * (CHUNK_WIDTH as i32));

        //Generates the mesh for this chunk, culling any faces of cube-shaped blocks that aren't touching a transparent block
        let mut north_vertices = Vec::new();
        let mut east_vertices = Vec::new();
        let mut south_vertices = Vec::new();
        let mut west_vertices = Vec::new();
        let mut up_vertices = Vec::new();
        let mut down_vertices = Vec::new();
        let mut other_vertices = Vec::new();

        for block_index in 0..CHUNK_VOLUME {
            let x = (block_index % CHUNK_WIDTH) as i32;
            let y = (block_index / CHUNK_AREA) as i16;
            let z = ((block_index % CHUNK_AREA) / CHUNK_WIDTH) as i32;

            let section_index = y / (CHUNK_SECTION_HEIGHT as i16);

            let block_state: ChunkBlockState = state_provider.get_state(x, y, z);

            if block_state.is_air() { continue; }

            let state_key = match block_state {
                ChunkBlockState::Air => unreachable!(),
                ChunkBlockState::State(key) => key,
            };

            if !filter(state_key) { continue; }

            let mesh = get_block(block_manager, block_state).unwrap();

            //TODO: randomly select a mesh if there are multiple

            match &mesh.models[0].0 {
                CubeOrComplexMesh::Cube(model) => {
                    let render_north = {
                        let state =
                            get_block(block_manager, state_provider.get_state(x, y, z - 1));

                        match state {
                            Some(mesh) => mesh.models[0].1,
                            None => true,
                        }
                    };

                    let render_south = {
                        let state =
                            get_block(block_manager, state_provider.get_state(x, y, z + 1));

                        match state {
                            Some(mesh) => mesh.models[0].1,
                            None => true,
                        }
                    };

                    let render_up = {
                        let state =
                            get_block(block_manager, state_provider.get_state(x, y + 1, z));

                        match state {
                            Some(mesh) => mesh.models[0].1,
                            None => true,
                        }
                    };

                    let render_down = {
                        let state =
                            get_block(block_manager, state_provider.get_state(x, y - 1, z));

                        match state {
                            Some(mesh) => mesh.models[0].1,
                            None => true,
                        }
                    };

                    let render_west = {
                        let state =
                            get_block(block_manager, state_provider.get_state(x - 1, y, z));

                        match state {
                            Some(mesh) => mesh.models[0].1,
                            None => true,
                        }
                    };

                    let render_east = {
                        let state =
                            get_block(block_manager, state_provider.get_state(x + 1, y, z));

                        match state {
                            Some(mesh) => mesh.models[0].1,
                            None => true,
                        }
                    };

                    if render_north || true {
                        match &model.north {
                            None => {}
                            Some(north) => north_vertices.extend(north.iter().map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk_world_x) as f32,
                                    y as f32,
                                    (z as i32 + chunk_world_z) as f32,
                                )
                            })),
                        };
                    }
                    if render_east || true {
                        match &model.east {
                            None => {}
                            Some(east) => east_vertices.extend(east.iter().map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk_world_x) as f32,
                                    y as f32,
                                    (z as i32 + chunk_world_z) as f32,
                                )
                            })),
                        };
                    }
                    if render_south || true {
                        match &model.south {
                            None => {}
                            Some(south) => south_vertices.extend(south.iter().map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk_world_x) as f32,
                                    y as f32,
                                    (z as i32 + chunk_world_z) as f32,
                                )
                            })),
                        };
                    }
                    if render_west || true {
                        match &model.west {
                            None => {}
                            Some(west) => west_vertices.extend(west.iter().map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk_world_x) as f32,
                                    y as f32,
                                    (z as i32 + chunk_world_z) as f32,
                                )
                            })),
                        };
                    }
                    if render_up || true {
                        match &model.up {
                            None => {}
                            Some(up) => up_vertices.extend(up.iter().map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk_world_x) as f32,
                                    y as f32,
                                    (z as i32 + chunk_world_z) as f32,
                                )
                            })),
                        };
                    }
                    if render_down || true {
                        match &model.down {
                            None => {}
                            Some(down) => down_vertices.extend(down.iter().map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk_world_x) as f32,
                                    y as f32,
                                    (z as i32 + chunk_world_z) as f32,
                                )
                            })),
                        };
                    }
                }
                CubeOrComplexMesh::Complex(model) => {
                    other_vertices.extend(
                        model
                            .iter()
                            .flat_map(|faces| {
                                [
                                    faces.north.as_ref(),
                                    faces.east.as_ref(),
                                    faces.south.as_ref(),
                                    faces.west.as_ref(),
                                    faces.up.as_ref(),
                                    faces.down.as_ref(),
                                ]
                            })
                            .flatten()
                            .flatten()
                            .map(|v| {
                                mapper(
                                    v,
                                    (x as i32 + chunk.pos.0) as f32,
                                    y as f32,
                                    (z as i32 + chunk.pos.1) as f32,
                                )
                            }),
                    );
                }
            }
        }

        Self {
            top: up_vertices,
            bottom: down_vertices,
            north: north_vertices,
            east: east_vertices,
            south: south_vertices,
            west: west_vertices,
            nonstandard: other_vertices,
        }
    }
}

impl<T: Copy + Pod> Default for BakedChunkLayer<T> {
    fn default() -> Self {
        Self::new()
    }
}
