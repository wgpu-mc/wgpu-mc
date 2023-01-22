use std::collections::HashMap;
use std::fmt::Debug;

use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::BufferUsages;

use crate::mc::block::{BlockMeshVertex, BlockstateKey, ChunkBlockState};
use crate::mc::BlockManager;
use crate::render::pipeline::Vertex;
use crate::render::world::chunk::bake;

use crate::WmRenderer;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 384;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;
pub const CHUNK_SECTION_HEIGHT: usize = 16;
pub const CHUNK_SECTIONS_PER: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

pub type ChunkPos = [i32; 2];

#[derive(Clone, Debug)]
pub struct ChunkSection {
    pub empty: bool,
    pub blocks: Box<[ChunkBlockState; SECTION_VOLUME]>,
    pub offset_y: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct LightLevel {
    pub level: u8
}

impl LightLevel {

    pub const fn from_sky_and_block(sky: u8, block: u8) -> Self {
        Self {
            level: (sky << 4) | (block & 0b1111),
        }
    }

    pub fn get_sky_level(&self) -> u8 {
        self.level >> 4
    }

    pub fn get_block_level(&self) -> u8 {
        self.level & 0b1111
    }

}

///Return a BlockState within the provided world coordinates.
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
pub struct Chunk {
    pub pos: ChunkPos,
    pub baked_layers: RwLock<HashMap<String, (wgpu::Buffer, Vec<Vertex>)>>,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            baked_layers: Default::default(),
        }
    }

    pub fn bake<T: BlockStateProvider>(
        &self,
        wm: &WmRenderer,
        layers: &[Box<dyn RenderLayer>],
        block_manager: &BlockManager,
        provider: &T,
    ) {
        let baked_layers = layers
            .iter()
            .map(|layer| {
                let verts = bake(
                    block_manager,
                    self,
                    layer.mapper(),
                    layer.filter(),
                    provider,
                );

                (
                    layer.name().into(),
                    (
                        wm.wgpu_state
                            .device
                            .create_buffer_init(&BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::cast_slice(&verts),
                                usage: BufferUsages::VERTEX,
                            }),
                        verts,
                    ),
                )
            })
            .collect();

        *self.baked_layers.write() = baked_layers;
    }
}

#[derive(Debug)]
pub struct ChunkManager {
    pub loaded_chunks: RwLock<HashMap<ChunkPos, ArcSwap<Chunk>>>,
    pub chunk_offset: Mutex<ChunkPos>,
}

impl ChunkManager {
    #[must_use]
    pub fn new() -> Self {
        ChunkManager {
            loaded_chunks: RwLock::new(HashMap::new()),
            chunk_offset: Mutex::new([0, 0]),
        }
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}
