//! # Everything regarding minecraft chunks
//!
//! This handles storing the state of all blocks in a chunk, as well as baking the chunk mesh
//!
//! # Chunk sections?
//!
//! Minecraft splits chunks into 16-block tall pieces called chunk sections, for
//! rendering purposes.
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;

use glam::IVec3;
use range_alloc::RangeAllocator;
use wgpu::BufferAddress;

use crate::mc::block::{BlockstateKey, ChunkBlockState, ModelMesh};
use crate::mc::BlockManager;
use crate::render::pipeline::Vertex;
use crate::WmRenderer;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 384;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;
pub const CHUNK_SECTION_HEIGHT: usize = 16;
pub const SECTIONS_PER_CHUNK: usize = CHUNK_HEIGHT / CHUNK_SECTION_HEIGHT;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

pub const MAX_CHUNKS: usize = 1000;

#[derive(Clone, Copy, Debug)]
pub struct LightLevel {
    pub byte: u8,
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
    fn get_state(&self, x: i32, y: i32, z: i32) -> ChunkBlockState;

    fn get_light_level(&self, x: i32, y: i32, z: i32) -> LightLevel;

    fn is_section_empty(&self, index: usize) -> bool;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum RenderLayer {
    Solid,
    Cutout,
    Transparent,
}

#[derive(Debug)]
pub struct SectionRanges {
    pub vertex_range: Range<u32>,
    pub index_range: Range<u32>,
    // #[cfg(not(feature = "vbo-fallback"))]
    // pub bind_group: wgpu::BindGroup,
}

///The struct representing a Chunk, with various render layers, split into sections
#[derive(Debug)]
pub struct Section {
    pub layers: HashMap<RenderLayer, SectionRanges>,
    pub pos: IVec3,
}

impl Section {
    pub fn new(pos: IVec3) -> Self {
        Self {
            layers: HashMap::new(),
            pos,
        }
    }

    pub fn bake_section<T: BlockStateProvider>(
        &mut self,
        wm: &WmRenderer,
        block_manager: &BlockManager,
        allocator: &mut RangeAllocator<u32>,
        buffer: Arc<wgpu::Buffer>,
        provider: &T,
    ) {
        let baked_layers = bake_section(self.pos, block_manager, provider);

        let mut layers = HashMap::new();

        for (layer, baked) in baked_layers {
            let mut vertex_data = Vec::new();
            let mut index_data = Vec::new();

            let vertex_size = baked.vertices.len() * size_of::<Vertex>();

            if vertex_size == 0 {
                continue;
            }

            let vertex_range = allocator.allocate_range(vertex_size.div_ceil(4) as u32).unwrap();
            let index_range = allocator.allocate_range(baked.indices.len() as u32).unwrap();

            dbg!(&vertex_range);
            dbg!(&index_range);

            vertex_data.extend(baked.vertices.iter().flat_map(Vertex::compressed));
            index_data.extend(baked.indices.iter().map(|index| (*index + vertex_range.start).to_ne_bytes()).flatten());

            wm.chunk_update_queue.0.send((buffer.clone(), vertex_data, vertex_range.start * 4)).unwrap();
            wm.chunk_update_queue.0.send((buffer.clone(), index_data, index_range.start * 4)).unwrap();

            layers.insert(
                layer,
                SectionRanges {
                    vertex_range: vertex_range.start * 4..vertex_range.end * 4,
                    index_range: index_range.start * 4..index_range.end * 4,
                }
            );
        }

        self.layers = layers;
    }
}

/// Returns true if the block at the given coordinates is either not a full cube or has transparency
#[inline]
fn block_allows_neighbor_render(
    block_manager: &BlockManager,
    state_provider: &impl BlockStateProvider,
    x: i32,
    y: i32,
    z: i32,
) -> bool {
    let state = get_block(block_manager, state_provider.get_state(x, y, z));
    match state {
        Some(mesh) => !mesh.is_cube,
        None => true,
    }
}

#[inline]
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
            .get_model(key.augment, 0),
    )
}

#[derive(Clone, Default)]
struct BakedSection {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn bake_section<Provider: BlockStateProvider>(
    pos: IVec3,
    block_manager: &BlockManager,
    state_provider: &Provider,
) -> HashMap<RenderLayer, BakedSection> {
    let mut layers = HashMap::new();
    layers.insert(RenderLayer::Solid, BakedSection::default());
    layers.insert(RenderLayer::Cutout, BakedSection::default());
    layers.insert(RenderLayer::Transparent, BakedSection::default());

    if state_provider.is_section_empty(pos.y as usize) {
        return layers;
    }

    for block_index in 0..16 * 16 * 16 {
        let x = block_index & 15;
        let y = block_index >> 8;
        let z = (block_index & 255) >> 4;

        let xf32 = x as f32;
        let yf32 = y as f32;
        let zf32 = z as f32;

        let block_state: ChunkBlockState = state_provider.get_state(x, y, z);

        let state_key = match block_state {
            ChunkBlockState::Air => continue,
            ChunkBlockState::State(key) => key,
        };

        let model_mesh = get_block(block_manager, block_state).unwrap();

        // TODO: randomly select a mesh if there are multiple models in a variant

        const INDICES: [u32; 6] = [1, 3, 0, 2, 3, 1];

        for model in &model_mesh.mesh {
            if model.cube {
                let baked_should_render_face = |x_: i32, y_: i32, z_: i32| {
                    block_allows_neighbor_render(block_manager, state_provider, x_, y_, z_)
                };
                let render_east = baked_should_render_face(x + 1, y, z);
                let render_west = baked_should_render_face(x - 1, y, z);
                let render_up = baked_should_render_face(x, y + 1, z);
                let render_down = baked_should_render_face(x, y - 1, z);
                let render_south = baked_should_render_face(x, y, z + 1);
                let render_north = baked_should_render_face(x, y, z - 1);

                let mut extend_vertices =
                    |layer: RenderLayer, index: u32, light_level: LightLevel| {
                        let baked_layer = layers.get_mut(&layer).unwrap();
                        let vec_index = baked_layer.vertices.len();

                        baked_layer
                            .vertices
                            .extend((index..index + 4).map(|vert_index| {
                                let model_vertex = model.vertices[vert_index as usize];

                                Vertex {
                                    position: [
                                        xf32 + model_vertex.position[0],
                                        yf32 + model_vertex.position[1],
                                        zf32 + model_vertex.position[2],
                                    ],
                                    uv: model_vertex.tex_coords,
                                    normal: model_vertex.normal,
                                    color: u32::MAX,
                                    uv_offset: 0,
                                    lightmap_coords: state_provider.get_light_level(x, y, z).byte,
                                    dark: false,
                                }
                            }));
                        baked_layer
                            .indices
                            .extend(INDICES.map(|index| index + (vec_index as u32)));
                    };

                // dbg!(absolute_x, absolute_z, render_up, render_down, render_north, render_south, render_west, render_east);

                //"face" contains offsets into the array containing the model vertices.
                //We use those offsets to get the relevant vertices, and add them into the chunk vertices.
                //We then add the starting offset into the vertices to the face indices so that they match up.
                if let (true, Some(face)) = (render_north, &model.north) {
                    let light_level: LightLevel = state_provider.get_light_level(x, y, z - 1);
                    extend_vertices(model_mesh.layer, *face, light_level);
                }

                if let (true, Some(face)) = (render_east, &model.east) {
                    let light_level: LightLevel = state_provider.get_light_level(x + 1, y, z);
                    extend_vertices(model_mesh.layer, *face, light_level);
                }

                if let (true, Some(face)) = (render_south, &model.south) {
                    let light_level: LightLevel = state_provider.get_light_level(x, y, z + 1);
                    extend_vertices(model_mesh.layer, *face, light_level);
                }

                if let (true, Some(face)) = (render_west, &model.west) {
                    let light_level: LightLevel = state_provider.get_light_level(x - 1, y, z);
                    extend_vertices(model_mesh.layer, *face, light_level);
                }

                if let (true, Some(face)) = (render_up, &model.up) {
                    let light_level: LightLevel = state_provider.get_light_level(x, y + 1, z);
                    extend_vertices(model_mesh.layer, *face, light_level);
                }

                if let (true, Some(face)) = (render_down, &model.down) {
                    let light_level: LightLevel = state_provider.get_light_level(x, y - 1, z);
                    extend_vertices(model_mesh.layer, *face, light_level);
                }
            } else {
                let light_level: LightLevel = state_provider.get_light_level(x, y, z);

                [
                    model.north,
                    model.east,
                    model.south,
                    model.west,
                    model.up,
                    model.down,
                ]
                .iter()
                .filter_map(|face| *face)
                .for_each(|index| {
                    let baked_layer = layers.get_mut(&model_mesh.layer).unwrap();
                    let vec_index = baked_layer.vertices.len();

                    baked_layer
                        .vertices
                        .extend((index..index + 4).map(|vert_index| {
                            let model_vertex = model.vertices[vert_index as usize];

                            Vertex {
                                position: [
                                    xf32 + model_vertex.position[0],
                                    yf32 + model_vertex.position[1],
                                    zf32 + model_vertex.position[2],
                                ],
                                uv: model_vertex.tex_coords,
                                normal: model_vertex.normal,
                                color: u32::MAX,
                                uv_offset: 0,
                                lightmap_coords: state_provider.get_light_level(x, y, z).byte,
                                dark: false,
                            }
                        }));
                    baked_layer
                        .indices
                        .extend(INDICES.map(|index| index + (vec_index as u32)));
                });
            }
        }
    }

    layers
}
