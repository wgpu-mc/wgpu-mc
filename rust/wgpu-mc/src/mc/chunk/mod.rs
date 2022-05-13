use crate::mc::block::{BlockPos, ChunkBlockState, BlockstateKey};
use std::collections::HashMap;

use crate::render::world::chunk::BakedChunkLayer;

use arc_swap::ArcSwap;
use parking_lot::RwLock;
use std::fmt::Debug;
use std::sync::Arc;

use crate::mc::BlockManager;
use crate::render::pipeline::terrain::TerrainVertex;

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

#[derive(Debug)]
pub struct ChunkLayers {
    glass: BakedChunkLayer<TerrainVertex>,
    terrain: BakedChunkLayer<TerrainVertex>,
}

///Return a BlockState within the provided (world) coordinates. If the coordinates are out of bounds,
/// an `Option<BlockstateKey>` value of None should be returned
pub trait BlockStateProvider: Send + Sync + Debug {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState;
}

#[derive(Debug)]
pub struct Chunk {
    pub pos: ChunkPos,
    pub state_provider: Box<dyn BlockStateProvider>,
    pub baked: ArcSwap<Option<ChunkLayers>>,
}

impl Chunk {
    pub fn new(pos: ChunkPos, state_provider: Box<dyn BlockStateProvider>) -> Self {
        Self {
            pos,
            state_provider,
            baked: ArcSwap::new(Arc::new(None)),
        }
    }

    pub fn blockstate_at_pos(&self, pos: BlockPos) -> ChunkBlockState {
        let x = pos.0 % 16;
        let y = pos.1 as i16;
        let z = pos.2 % 16;

        self.state_provider.get_state(x, y, z)
    }

    pub fn bake(&self, block_manager: &BlockManager) {
        let glass_index: BlockstateKey = (*block_manager
            .block_key_indices
            .get("Block{minecraft:glass}")
            .unwrap() as u32)
            .into();

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
            },
            Box::new(move |state| match state.packed_key {
                None => false,
                Some(key) => key == glass_index,
            }),
            &*self.state_provider,
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
            },
            Box::new(move |state| match state.packed_key {
                None => false,
                Some(key) => key != glass_index,
            }),
            &*self.state_provider,
        );

        self.baked
            .store(Arc::new(Some(ChunkLayers { glass, terrain })));
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

    pub fn bake_meshes(&self, wm: &WmRenderer) {
        let block_manager = wm.mc.block_manager.read();

        use rayon::iter::ParallelIterator;
        self.loaded_chunks
            .read()
            .iter()
            .map(|(_pos, chunk)| {
                chunk.load().bake(&block_manager);
            })
            .collect::<Vec<_>>();
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
            let layers = (**baked).as_ref().unwrap();

            glass.extend(&layers.glass);
            terrain.extend(&layers.terrain);
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
