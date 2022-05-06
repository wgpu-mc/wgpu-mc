use crate::model::MeshVertex;
use crate::mc::chunk::{CHUNK_AREA, CHUNK_WIDTH, CHUNK_SECTION_HEIGHT, Chunk, CHUNK_VOLUME, WorldBuffers, BlockStateProvider};

use bytemuck::Pod;
use crate::mc::block::{BlockState};

use wgpu::util::{DeviceExt, BufferInitDescriptor};

use crate::mc::block::model::{CubeOrComplexMesh, BlockstateVariantMesh};

use crate::mc::BlockManager;
use crate::WmRenderer;

fn get_block_mesh<'a>(block_manager: &'a BlockManager, state: &BlockState) -> Option<&'a BlockstateVariantMesh> {
    (&block_manager.block_state_variants).get((*state).packed_key? as usize)
}

#[derive(Debug)]
pub struct BakedChunkLayer<T: Copy + Pod> {
    pub top: Vec<T>,
    pub bottom: Vec<T>,
    pub north: Vec<T>,
    pub east: Vec<T>,
    pub south: Vec<T>,
    pub west: Vec<T>,
    pub nonstandard: Vec<T>
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
            nonstandard: vec![]
        }
    }

    pub fn upload(&self, wm: &WmRenderer) -> WorldBuffers {
        WorldBuffers {
            top: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.top[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.top.len()
            ),
            bottom: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.bottom[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.bottom.len()
            ),
            north: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.north[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.north.len()
            ),
            south: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.south[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.south.len()
            ),
            west: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.west[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.west.len()
            ),
            east: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.east[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.east.len()
            ),
            other: (
                wm.wgpu_state.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&self.nonstandard[..]),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                ), self.nonstandard.len()
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
    pub fn bake(block_manager: &BlockManager, chunk: &Chunk, mapper: fn (&MeshVertex, f32, f32, f32) -> T, filter: Box<dyn Fn(BlockState) -> bool>, state_provider: &dyn BlockStateProvider) -> Self {
        //Generates the mesh for this chunk, hiding any full-block faces that aren't touching a transparent block
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
            let z  = ((block_index % CHUNK_AREA) / CHUNK_WIDTH) as i32;

            let section_index = y / (CHUNK_SECTION_HEIGHT as i16);

            let block_state: BlockState = state_provider.get_state(x, y, z);

            if !filter(block_state) { continue; }

            let baked_mesh = match get_block_mesh(block_manager, &block_state) {
                None => continue,
                Some(mesh) => mesh,
            };

            match &baked_mesh.shape {
                CubeOrComplexMesh::Cube(model) => {
                    let render_north = {
                        let block_mesh = get_block_mesh(
                            block_manager,
                            &state_provider.get_state(x, y, z - 1)
                        );

                        match block_mesh {
                            Some(mesh) => mesh.transparent_or_complex,
                            None => true,
                        }
                    };

                    let render_south = {
                        let block_mesh = get_block_mesh(
                            block_manager,
                            &state_provider.get_state(x, y, z + 1)
                        );

                        match block_mesh {
                            Some(mesh) => mesh.transparent_or_complex,
                            None => true,
                        }
                    };

                    let render_up = {
                        let block_mesh = get_block_mesh(
                            block_manager,
                            &state_provider.get_state(x, y + 1, z)
                        );

                        match block_mesh {
                            Some(mesh) => mesh.transparent_or_complex,
                            None => true,
                        }
                    };

                    let render_down = {
                        let block_mesh = get_block_mesh(
                            block_manager,
                            &state_provider.get_state(x, y - 1, z)
                        );

                        match block_mesh {
                            Some(mesh) => mesh.transparent_or_complex,
                            None => true,
                        }
                    };

                    let render_west = {
                        let block_mesh = get_block_mesh(
                            block_manager,
                            &state_provider.get_state(x - 1, y, z)
                        );

                        match block_mesh {
                            Some(mesh) => mesh.transparent_or_complex,
                            None => true,
                        }
                    };

                    let render_east = {
                        let block_mesh = get_block_mesh(
                            block_manager,
                            &state_provider.get_state(x + 1, y, z)
                        );

                        match block_mesh {
                            Some(mesh) => mesh.transparent_or_complex,
                            None => true,
                        }
                    };

                    if render_north {
                        match &model.north {
                            None => {}
                            Some(north) =>
                                north_vertices.extend(north.iter().map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32)))
                        };
                    }
                    if render_east {
                        match &model.east {
                            None => {}
                            Some(east) =>
                                east_vertices.extend(east.iter().map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32)))
                        };
                    }
                    if render_south {
                        match &model.south {
                            None => {}
                            Some(south) =>
                                south_vertices.extend(south.iter().map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32)))
                        };
                    }
                    if render_west {
                        match &model.west {
                            None => {}
                            Some(west) =>
                                west_vertices.extend(west.iter().map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32)))
                        };
                    }
                    if render_up {
                        match &model.up {
                            None => {}
                            Some(up) =>
                                up_vertices.extend(up.iter().map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32)))
                        };
                    }
                    if render_down {
                        match &model.down {
                            None => {}
                            Some(down) =>
                                down_vertices.extend(down.iter().map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32)))
                        };
                    }
                }
                CubeOrComplexMesh::Custom(model) => {
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
                            .flatten()
                            .flatten()
                            .map(|v| mapper(v, (x as i32 + chunk.pos.0) as f32, y as f32, (z as i32 + chunk.pos.1) as f32))
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
            nonstandard: other_vertices
        }
    }
}

impl<T: Copy + Pod> Default for BakedChunkLayer<T> {
    fn default() -> Self {
        Self::new()
    }
}