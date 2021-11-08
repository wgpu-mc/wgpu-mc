use crate::mc::block::{Block, BlockPos, BlockState, BlockShape};
use crate::model::MeshVertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::render::chunk::BakedChunk;
use parking_lot::RwLock;
use std::sync::Arc;
use std::convert::TryInto;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;
pub const CHUNK_SECTION_HEIGHT: usize = 1;
pub const CHUNK_SECTIONS_PER: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

type ChunkPos = (i32, i32);

#[derive(Clone, Debug)]
pub struct ChunkSection {
    pub empty: bool,
    pub blocks: Box<[BlockState; SECTION_VOLUME]>,
    pub offset_y: usize
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
    pub baked: Option<BakedChunk>
}

impl Chunk {
    #[must_use]
    pub fn new(pos: ChunkPos, mut blocks: Box<[BlockState; CHUNK_AREA * CHUNK_HEIGHT]>) -> Self {
        let sections: Box<[ChunkSection; CHUNK_SECTIONS_PER]> = (0..CHUNK_SECTIONS_PER).map(|section| {
            let start_index = section * SECTION_VOLUME;
            let end_index = (section + 1) * SECTION_VOLUME;
            let block_section: Box<[BlockState; SECTION_VOLUME]> = (start_index..end_index).map(|index| {
                blocks[index]
            }).collect::<Box<[BlockState]>>().try_into().unwrap();

            ChunkSection {
                empty: !blocks.iter().any(|state| state.block.is_some()),
                blocks: block_section,
                offset_y: section * CHUNK_SECTION_HEIGHT
            }
        }).collect::<Box<[ChunkSection]>>().try_into().unwrap();

        Self {
            pos,
            sections,
            baked: None
        }
    }

    #[must_use]
    pub fn blockstate_at_pos(&self, pos: BlockPos) -> BlockState {
        let x = (pos.0 % 16) as usize;
        let y = (pos.1) as usize;
        let z = (pos.2 % 16) as usize;

        self.sections[y].blocks[(z * CHUNK_WIDTH) + x]
    }
}

pub struct ChunkManager {
    //Due to floating point inaccuracy at large distances,
    //we need to keep the model coordinates as close to 0,0,0 as possible
    pub chunk_origin: ChunkPos,
    pub loaded_chunks: Vec<RwLock<Chunk>>,
}

impl ChunkManager {
    #[must_use]
    pub fn new() -> Self {
        ChunkManager {
            chunk_origin: (0, 0),
            loaded_chunks: Vec::new(),
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
