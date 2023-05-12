//! # Everything regarding minecraft chunks
//!
//! This handles storing the state of all blocks in a chunk, as well as baking the chunk mesh
//!
//! # Chunk sections?
//!
//! Minecraft splits chunks into 16-block tall pieces called chunk sections, for
//! rendering purposes.

use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use range_alloc::RangeAllocator;
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferAddress, BufferDescriptor, BufferUsages};

use crate::mc::block::{
    BlockMeshVertex, BlockstateKey, ChunkBlockState, CubeOrComplexMesh, ModelMesh,
};
use crate::mc::BlockManager;
use crate::render::pipeline::Vertex;

use crate::{WgpuState, WmRenderer};
use crate::util::BindableBuffer;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 384;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;
pub const CHUNK_SECTION_HEIGHT: usize = 16;
pub const SECTIONS_PER_CHUNK: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

pub const CHUNK_ALLOCATOR_SIZE: usize = 1024 * 1024 * 1024;

pub type ChunkPos = [i32; 2];

pub struct ChunkManager {
    pub loaded_chunks: RwLock<HashMap<ChunkPos, ArcSwap<Chunk>>>,
    pub chunk_offset: Mutex<ChunkPos>,
}

impl ChunkManager {
    #[must_use]
    pub fn new(wgpu_state: &WgpuState) -> Self {
        ChunkManager {
            loaded_chunks: RwLock::new(HashMap::new()),
            chunk_offset: Mutex::new([0, 0]),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LightLevel {
    pub byte: u8
}

impl LightLevel {

    pub const fn from_sky_and_block(sky: u8, block: u8) -> Self {
        Self {
            byte: (sky << 4) | (block & 0b1111),
        }
    }

    pub fn get_sky_level(&self) -> u8 {
        self.byte >> 4
    }

    pub fn get_block_level(&self) -> u8 {
        self.byte & 0b1111
    }

}

/// Return a [ChunkBlockState] within the provided world coordinates.
pub trait BlockStateProvider: Send + Sync {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState;

    fn get_light_level(&self, x: i32, y: i16, z: i32) -> LightLevel;

    fn is_section_empty(&self, index: usize) -> bool;
}

pub trait RenderLayer: Send + Sync {
    fn filter(&self) -> fn(BlockstateKey) -> bool;

    fn mapper(&self) -> fn(&BlockMeshVertex, f32, f32, f32, LightLevel) -> Vertex;

    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct ChunkBuffers {
    pub vertex_buffer: BindableBuffer,
    pub index_buffer: BindableBuffer
}

///The struct representing a Chunk, with various render layers, split into sections
#[derive(Debug)]
pub struct Chunk {
    pub pos: ChunkPos,
    pub buffers: ArcSwap<Option<ChunkBuffers>>,
    pub sections: RwLock<HashMap<String, [Range<u32>; SECTIONS_PER_CHUNK]>>,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            sections: RwLock::new(HashMap::new()),
            buffers: ArcSwap::new(Arc::new(None)),
        }
    }

    /// Bakes the layers, and uploads them to the GPU.
    pub fn bake_chunk<T: BlockStateProvider>(
        &self,
        wm: &WmRenderer,
        layers: &[Box<dyn RenderLayer>],
        block_manager: &BlockManager,
        provider: &T,
    ) {
        let mut vertices = 0;
        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();

        let baked_layers = layers
            .iter()
            .map(|layer| {
                let sections = (0..SECTIONS_PER_CHUNK).map(|index| {
                    bake_section_layer(
                        block_manager,
                        self,
                        layer.mapper(),
                        layer.filter(),
                        provider,
                        index
                    )
                }).collect::<Vec<_>>();

                let ranges: [Range<u32>; SECTIONS_PER_CHUNK] = array_init::from_iter(
                    sections.iter().map(|(vert, index)| {
                        let offset = vertices as u32;
                        vertices += vert.len();

                        let index_offset = index_data.len();

                        vertex_data.extend(vert.iter().map(Vertex::compressed).flatten());
                        index_data.extend(index.iter().map(|i| *i + offset));

                        index_offset as u32..index_offset as u32 + index.len() as u32
                    })
                ).unwrap();

                (
                    layer.name().to_string(),
                    ranges
                )
            })
            .collect();

        let vertex_buffer = BindableBuffer::new(wm, &vertex_data, BufferUsages::STORAGE, "ssbo");
        let index_buffer = BindableBuffer::new(wm, bytemuck::cast_slice(&index_data), BufferUsages::STORAGE, "ssbo");

        self.buffers.store(Arc::new(Some(ChunkBuffers {
            vertex_buffer,
            index_buffer,
        })));
        *self.sections.write() = baked_layers;
    }
}

/// Returns true if the block at the given coordinates is either not a full cube or has transparency
#[inline]
fn is_block_not_fully_opaque(
    block_manager: &BlockManager,
    state_provider: &impl BlockStateProvider,
    x: i32,
    y: i16,
    z: i32,
) -> bool {
    let state = get_block(block_manager, state_provider.get_state(x, y, z));

    match state {
        Some(mesh) => mesh.models[0].1,
        None => true,
    }
}

fn get_block(block_manager: &BlockManager, state: ChunkBlockState) -> Option<Arc<ModelMesh>> {
    let key = match state {
        ChunkBlockState::Air => return None,
        ChunkBlockState::State(key) => key,
    };

    Some(
        block_manager
            .blocks
            .get_index(key.block as usize)?
            .1
            .get_model(key.augment),
    )
}

pub fn bake_section_layer<
    T,
    Provider: BlockStateProvider,
    Filter: Fn(BlockstateKey) -> bool,
    Mapper: Fn(&BlockMeshVertex, f32, f32, f32, LightLevel) -> T,
>(
    block_manager: &BlockManager,
    chunk: &Chunk,
    mapper: Mapper,
    filter: Filter,
    state_provider: &Provider,
    section_index: usize
) -> (Vec<T>, Vec<u32>) {
    //Generates the mesh for this chunk, culling faces whenever possible
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut block_index = section_index * (16 * 16 * 16);

    let end_index = (section_index + 1) * (16 * 16 * 16);

    if state_provider.is_section_empty(section_index) {
        return (vertices, indices);
    }

    loop {
        if block_index >= end_index {
            break;
        }

        let x = (block_index % CHUNK_WIDTH) as i32;
        let y = (block_index / CHUNK_AREA) as i16;
        let z = ((block_index % CHUNK_AREA) / CHUNK_WIDTH) as i32;

        let xf32 = x as f32;
        let yf32 = (y % 16) as f32;
        let zf32 = z as f32;

        block_index += 1;

        let absolute_x = (chunk.pos[0] * 16) + x;
        let absolute_z = (chunk.pos[1] * 16) + z;

        let block_state: ChunkBlockState = state_provider.get_state(absolute_x, y, absolute_z);
        let light_level: LightLevel = state_provider.get_light_level(absolute_x, y, absolute_z);

        if block_state.is_air() {
            continue;
        }

        let state_key = match block_state {
            ChunkBlockState::Air => unreachable!(),
            ChunkBlockState::State(key) => key,
        };

        if !filter(state_key) {
            continue;
        }

        let mesh = get_block(block_manager, block_state).unwrap();

        // TODO: randomly select a mesh if there are multiple

        match &mesh.models[0].0 {
            CubeOrComplexMesh::Cube(model) => {
                let baked_should_render_face = |x_: i32, y_: i16, z_: i32| {
                    is_block_not_fully_opaque(block_manager, state_provider, x_, y_, z_)
                };

                let render_east = baked_should_render_face(absolute_x + 1, y, absolute_z);
                let render_west = baked_should_render_face(absolute_x - 1, y, absolute_z);
                let render_up = baked_should_render_face(absolute_x, y + 1, absolute_z);
                let render_down = baked_should_render_face(absolute_x, y - 1, absolute_z);
                let render_south = baked_should_render_face(absolute_x, y, absolute_z + 1);
                let render_north = baked_should_render_face(absolute_x, y, absolute_z - 1);

                let add_face = || {
                    render_east
                };

                const INDICES: [u32; 6] = [0,1,2,3,4,5];

                let mut extend_vertices = |face: &[u32; 6]| {
                    let vec_index = vertices.len();
                    vertices.extend(
                        face.map(|index| mapper(&model.vertices[index as usize], xf32, yf32, zf32, light_level))
                    );
                    indices.extend(INDICES.map(|index| index + (vec_index as u32)));
                };

                //"face" contains offsets into the array containing the model vertices.
                //We use those offsets to get the relevant vertices, and add them into the chunk vertices.
                //We then add the starting offset into the vertices to the face indices so that they match up.
                if let (true, Some(face)) = (render_north, &model.north) {
                    extend_vertices(face);
                }

                if let (true, Some(face)) = (render_east, &model.east) {
                    extend_vertices(face);
                }

                if let (true, Some(face)) = (render_south, &model.south) {
                    extend_vertices(face);
                }

                if let (true, Some(face)) = (render_west, &model.west) {
                    extend_vertices(face);
                }

                if let (true, Some(face)) = (render_up, &model.up) {
                    extend_vertices(face);
                }

                if let (true, Some(face)) = (render_down, &model.down) {
                    extend_vertices(face);
                }
            }
            CubeOrComplexMesh::Complex(faces) => {
                for model in faces {
                    let unwrapped_faces = [
                        model.north,
                        model.east,
                        model.south,
                        model.west,
                        model.up,
                        model.down
                    ].iter().filter_map(|face| *face).collect::<Vec<_>>();

                    vertices.extend(unwrapped_faces.iter().flatten().map(|index| mapper(&model.vertices[*index as usize], xf32, yf32, zf32, light_level)));
                    indices.extend(unwrapped_faces.iter().flatten());
                }
            }
        }
    }

    (vertices, indices)
}
