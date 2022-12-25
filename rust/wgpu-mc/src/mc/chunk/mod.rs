use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use arc_swap::ArcSwap;
use get_size::GetSize;
use parking_lot::RwLock;
use rayon::iter::IntoParallelRefIterator;
use wgpu::BufferUsages;

use crate::mc::block::{BlockMeshVertex, BlockstateKey, ChunkBlockState};
use crate::mc::BlockManager;
use crate::render::pipeline::terrain::Vertex;
use crate::render::world::chunk;
use crate::render::world::chunk::{bake, BakedChunkLayer};
use crate::util::SSBO;
use crate::WmRenderer;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 384;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;
pub const CHUNK_SECTION_HEIGHT: usize = 1;
pub const CHUNK_SECTIONS_PER: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

pub type ChunkPos = [i32; 2];

#[derive(Clone, Debug)]
pub struct ChunkSection {
    pub empty: bool,
    pub blocks: Box<[ChunkBlockState; SECTION_VOLUME]>,
    pub offset_y: usize,
}

///Return a BlockState within the provided world coordinates.
pub trait BlockStateProvider: Send + Sync + Debug {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState;
}

pub trait RenderLayer {

    fn filter(&self) -> fn(BlockstateKey) -> bool;

    fn mapper(&self) -> fn(&BlockMeshVertex, f32, f32, f32) -> Vertex;

    fn name(&self) -> &str;

}

#[derive(Debug)]
pub struct Chunk {
    pub pos: ChunkPos,
    pub pos_buffer: SSBO,
    pub baked_layers: RwLock<HashMap<String, (SSBO, Vec<Vertex>)>>,
}

impl Chunk {
    pub fn new(pos: ChunkPos, wm: &WmRenderer) -> Self {
        Self {
            pos,
            pos_buffer: SSBO::new(wm, bytemuck::cast_slice(&pos), BufferUsages::STORAGE, false),
            baked_layers: Default::default(),
        }
    }

    pub fn bake<T: BlockStateProvider>(&self, wm: &WmRenderer, layers: &[Box<dyn RenderLayer>], block_manager: &BlockManager, provider: &T, ) {
        let baked_layers = layers.par_iter().map(|layer| {
            let verts = bake(block_manager, &self, layer.mapper(), layer.filter(), provider);

            (layer.name().into(), (SSBO::new(wm, bytemuck::cast_slice(&verts), BufferUsages::VERTEX, false)))
        }).collect();

        *self.baked_layers.write() = baked_layers;
    }
}

impl GetSize for Chunk {
    fn get_heap_size(&self) -> usize {
        GetSize::get_size(&self.baked_layers.load_full())
    }
}

pub struct ChunkManager {
    pub loaded_chunks: RwLock<HashMap<ChunkPos, ArcSwap<Chunk>>>,
}

impl ChunkManager {
    #[must_use]
    pub fn new() -> Self {
        ChunkManager {
            loaded_chunks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}
