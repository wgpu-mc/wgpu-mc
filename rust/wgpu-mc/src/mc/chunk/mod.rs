use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::RwLock;

use crate::mc::block::{BlockstateKey, ChunkBlockState};
use crate::mc::BlockManager;
use crate::render::pipeline::terrain::TerrainVertex;
use crate::render::world::chunk::BakedChunkLayer;

use get_size::GetSize;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 384;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;
pub const CHUNK_SECTION_HEIGHT: usize = 1;
pub const CHUNK_SECTIONS_PER: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

use crate::WmRenderer;

pub type ChunkPos = (i32, i32);

#[derive(Clone, Debug)]
pub struct ChunkSection {
    pub empty: bool,
    pub blocks: Box<[ChunkBlockState; SECTION_VOLUME]>,
    pub offset_y: usize,
}

pub struct RenderLayers {
    pub terrain: Box<[ChunkSection; CHUNK_SECTIONS_PER]>,
    pub transparent: Box<[ChunkSection; CHUNK_SECTIONS_PER]>,
    pub grass: Box<[ChunkSection; CHUNK_SECTIONS_PER]>,
}

#[derive(Debug, GetSize)]
pub struct ChunkLayers {
    glass: BakedChunkLayer<TerrainVertex>,
    terrain: BakedChunkLayer<TerrainVertex>,
}

///Return a BlockState within the provided world coordinates.
pub trait BlockStateProvider: Send + Sync + Debug {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState;
}

#[derive(Debug)]
pub struct Chunk {
    pub pos: ChunkPos,
    pub baked: ArcSwap<Option<ChunkLayers>>,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            baked: ArcSwap::new(Arc::new(None)),
        }
    }

    pub fn bake<T: BlockStateProvider>(&self, block_manager: &BlockManager, provider: &T) {
        let glass_state = BlockstateKey {
            block: block_manager.blocks.get_full("minecraft:glass").unwrap().0 as u16,
            augment: 0,
        };

        let glass = BakedChunkLayer::bake(
            block_manager,
            self,
            |v, x, y, z| TerrainVertex {
                position: [v.position[0] + x, v.position[1] + y, v.position[2] + z],
                tex_coords: v.tex_coords,
                lightmap_coords: [0.0, 0.0],
                normal: [v.normal[0], v.normal[1], v.normal[2], 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tangent: [1.0, 1.0, 1.0, 1.0],
                uv_offset: v.animation_uv_offset,
            },
            Box::new(move |state| state == glass_state),
            provider,
        );

        let terrain = BakedChunkLayer::bake(
            block_manager,
            self,
            |v, x, y, z| TerrainVertex {
                position: [v.position[0] + x, v.position[1] + y, v.position[2] + z],
                tex_coords: v.tex_coords,
                lightmap_coords: [0.0, 0.0],
                normal: [v.normal[0], v.normal[1], v.normal[2], 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tangent: [1.0, 1.0, 1.0, 1.0],
                uv_offset: v.animation_uv_offset,
            },
            Box::new(move |state| state != glass_state),
            provider,
        );

        self.baked
            .store(Arc::new(Some(ChunkLayers { glass, terrain })));
    }
}

impl GetSize for Chunk {

    fn get_heap_size(&self) -> usize {
        GetSize::get_size(&self.baked.load_full())
    }

}

pub struct WorldBuffers {
    pub top: (wgpu::Buffer, usize),
    pub bottom: (wgpu::Buffer, usize),
    pub north: (wgpu::Buffer, usize),
    pub south: (wgpu::Buffer, usize),
    pub west: (wgpu::Buffer, usize),
    pub east: (wgpu::Buffer, usize),
    pub other: (wgpu::Buffer, usize),
}

pub struct ChunkManager {
    //Due to floating point inaccuracy at large distances,
    //we need to keep the model coordinates as close to 0,0,0 as possible
    pub chunk_origin: ArcSwap<ChunkPos>,
    pub loaded_chunks: RwLock<HashMap<ChunkPos, ArcSwap<Chunk>>>,
    pub section_buffers: ArcSwap<HashMap<String, WorldBuffers>>,
}

impl ChunkManager {
    #[must_use]
    pub fn new() -> Self {
        ChunkManager {
            chunk_origin: ArcSwap::new(Arc::new((0, 0))),
            loaded_chunks: RwLock::new(HashMap::new()),
            section_buffers: ArcSwap::new(Arc::new(HashMap::new())),
        }
    }

    pub fn bake_meshes<T: BlockStateProvider>(&self, wm: &WmRenderer, provider: &T) {
        let block_manager = wm.mc.block_manager.read();
        self.loaded_chunks.read().iter().for_each(|(_pos, chunk)| {
            chunk.load().bake(&block_manager, provider);
        });
    }

    pub fn assemble_world_meshes(&self, wm: &WmRenderer) {
        let chunks = self
            .loaded_chunks
            .read()
            .iter()
            .map(|chunk| chunk.1.load_full())
            .collect::<Vec<_>>();

        let mut glass = BakedChunkLayer::new();
        let mut terrain = BakedChunkLayer::new();

        chunks.iter().for_each(|chunk| {
            let baked = chunk.baked.load();

            match &(**baked) {
                Some(layers) => {
                    glass.extend(&layers.glass);
                    terrain.extend(&layers.terrain);
                }
                None => {}
            };
        });

        let mut map = HashMap::new();

        map.insert("transparent".into(), glass.upload(wm));
        map.insert("terrain".into(), terrain.upload(wm));

        self.section_buffers.store(Arc::new(map));
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}
