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
use std::ops::{Not, Range};
use std::sync::Arc;
use arrayvec::ArrayVec;
use get_size::GetSize;
use glam::{ivec3, vec3, IVec2, IVec3, Vec3Swizzles};
use range_alloc::RangeAllocator;

use crate::mc::block::{BlockModelFace, ChunkBlockState, ModelMesh};
use crate::mc::direction::Direction;
use crate::mc::BlockManager;
use crate::render::pipeline::Vertex;
use crate::WmRenderer;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = 384;
pub const CHUNK_SECTION_HEIGHT: usize = 16;
pub const SECTION_VOLUME: usize = CHUNK_AREA * CHUNK_SECTION_HEIGHT;

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
    fn get_state(&self, pos: IVec3) -> ChunkBlockState;

    fn get_light_level(&self, pos: IVec3) -> LightLevel;

    fn is_section_empty(&self, rel_pos: IVec3) -> bool;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum RenderLayer {
    Solid = 0,
    Cutout = 1,
    Transparent = 2,
}

#[derive(Clone)]
pub struct SectionRanges {
    pub vertex_range: Range<u32>,
    pub index_range: Range<u32>,
}

///The struct representing a Chunk section, with various render layers, split into sections
pub struct SectionStorage {
    storage: HashMap<IVec3, Section>,
    allocator: RangeAllocator<u32>,
    width: i32,
}
impl SectionStorage {
    pub fn new(range: u32) -> Self {
        SectionStorage {
            storage: HashMap::new(),
            width: 0,
            allocator: RangeAllocator::new(0..range),
        }
    }
    pub fn clear(&mut self) {
        self.allocator.reset();
        self.storage.clear();
    }
    pub fn set_width(&mut self, w: i32) {
        self.width = w;
    }
    pub fn trim(&mut self, pos: IVec2) {
        let mut to_remove = vec![];
        for (k, section) in &self.storage {
            let dist = (k.xz() - pos).abs();
            let radius = self.width + 2; //temp fix until proper sync
            if dist.x > radius || dist.y > radius {
                to_remove.push(*k);
                for layer in &section.layers {
                    if let Some(l) = layer.as_ref() {
                        self.allocator.free_range(l.vertex_range.clone());
                        self.allocator.free_range(l.index_range.clone());
                    }
                }
            }
        }
        to_remove.iter().for_each(|pos| {
            self.storage.remove(pos);
        });
    }
    pub fn replace(&mut self, pos: IVec3, baked_layers: &Vec<BakedLayer>) -> Section {
        if let Some(previous_section) = self.storage.get(&pos) {
            for layer in &previous_section.layers {
                if let Some(l) = layer.as_ref() {
                    self.allocator.free_range(l.vertex_range.clone());
                    self.allocator.free_range(l.index_range.clone());
                }
            }
        }
        let section = Section {
            layers: baked_layers
                .iter()
                .map(|layer| {
                    if !layer.indices.is_empty() {
                        Some(SectionRanges {
                            vertex_range: self
                                .allocator
                                .allocate_range(layer.vertices.len() as u32 / 4)
                                .unwrap(),
                            index_range: self
                                .allocator
                                .allocate_range(layer.indices.len() as u32 / 4)
                                .unwrap(),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        };
        self.storage.insert(pos, section.clone());
        section
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<IVec3, Section> {
        self.storage.iter()
    }
}

#[derive(Clone)]
pub struct Section {
    pub layers: Vec<Option<SectionRanges>>,
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

impl Section {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }
}

#[inline]
fn get_block(block_manager: &BlockManager, state: ChunkBlockState) -> Option<Arc<ModelMesh>> {
    let key = match state {
        ChunkBlockState::Air => return None,
        ChunkBlockState::State(key) => key,
    };
    
    block_manager
        .blocks
        .get_index(key.block as usize)?
        .1
        .get_model(key.augment, 0)
}

pub fn bake_section<Provider: BlockStateProvider>(pos: IVec3, wm: &WmRenderer, bsp: &Provider) {
    let bm = wm.mc.block_manager.read();

    let baked_section = bake_layers(pos, &bm, bsp);

    wm.chunk_update_queue.0.send((pos, baked_section)).unwrap();
}

#[derive(Clone, Default)]
pub struct BakedLayer {
    pub vertices: Vec<u8>,
    pub indices: Vec<u8>,
}

fn bake_layers<Provider: BlockStateProvider>(
    pos: IVec3,
    block_manager: &BlockManager,
    state_provider: &Provider,
) -> Vec<BakedLayer> {
    let mut layers = vec![BakedLayer::default(); 3];

    if state_provider.is_section_empty(ivec3(0, 0, 0)) {
        return layers;
    }

    for block_index in 0..16 * 16 * 16 {
        let pos = ivec3(block_index & 15, block_index >> 8, (block_index & 255) >> 4);

        let fpos = vec3(pos.x as f32, pos.y as f32, pos.z as f32);

        let block_state: ChunkBlockState = state_provider.get_state(pos);

        if let Some(model_mesh) = get_block(block_manager, block_state) {
            const INDICES: [u32; 6] = [1, 3, 0, 2, 3, 1];
            let mut add_quad = |face: &BlockModelFace, light_level: LightLevel, dir: Direction| {
                let baked_layer = &mut layers[RenderLayer::Solid as usize];
                let vec_index = baked_layer.vertices.len() / Vertex::VERTEX_LENGTH;

                let dir_vec = dir.to_vec();
                
                baked_layer.vertices.extend(
                    (0..4)
                        .map(|vert_index| {
                            let model_vertex = face.vertices[vert_index as usize];

                            let (b1, b2, b3) = if model_mesh.any.is_empty() {
                                let vertex_biases = ivec3(
                                    if model_vertex.position.x as i32 == 0 {
                                        -1
                                    } else {
                                        1
                                    },
                                    if model_vertex.position.y as i32 == 0 {
                                        -1
                                    } else {
                                        1
                                    },
                                    if model_vertex.position.z as i32 == 0 {
                                        -1
                                    } else {
                                        1
                                    },
                                );

                                let axis = dir_vec - vertex_biases; //equivalent to -(vertex_biases - dir_vec)

                                let mut axes: ArrayVec<IVec3, 2> = ArrayVec::new_const();

                                if axis.x != 0 {
                                    axes.push(ivec3(axis.x, 0 ,0));
                                }

                                if axis.y != 0 {
                                    axes.push(ivec3(0, axis.y,0));
                                }

                                if axis.z != 0 {
                                    axes.push(ivec3(0, 0 ,axis.z));
                                }

                                let p1 = vertex_biases + pos;
                                let p2 = p1 + axes[0];
                                let p3 = p1 + axes[1];

                                let b1 = state_provider.get_state(p1).is_air().not() as u8;
                                let b2 = state_provider.get_state(p2).is_air().not() as u8;
                                let b3 = state_provider.get_state(p3).is_air().not() as u8;

                                (b1, b2, b3)
                            } else {
                                (0, 0, 0)
                            };
                            
                            Vertex {
                                position: [
                                    fpos.x + model_vertex.position[0],
                                    fpos.y + model_vertex.position[1],
                                    fpos.z + model_vertex.position[2],
                                ],
                                uv: model_vertex.tex_coords,
                                normal: face.normal.to_array(),
                                color: u32::MAX,
                                uv_offset: 0,
                                lightmap_coords: light_level.byte,
                                ao: 3 - (b1 + b2 + b3),
                            }
                        })
                        .flat_map(Vertex::compressed),
                );
                baked_layer.indices.extend(
                    INDICES
                        .iter()
                        .flat_map(|index| (index + (vec_index as u32)).to_ne_bytes()),
                );
            };

            let mut add_face = |face: &BlockModelFace, dir: Direction| {
                let cull = if let Some(mesh) =
                    get_block(block_manager, state_provider.get_state(pos + dir.to_vec()))
                {
                    (mesh.cull >> dir.opposite() as u8) & 1 == 1
                } else {
                    false
                };

                if !cull {
                    let light_level: LightLevel =
                        state_provider.get_light_level(pos + dir.to_vec());
                    add_quad(face, light_level, dir);
                }
            };

            model_mesh.west.iter().for_each(|face| {
                add_face(face, Direction::West);
            });
            model_mesh.east.iter().for_each(|face| {
                add_face(face, Direction::East);
            });
            model_mesh.down.iter().for_each(|face| {
                add_face(face, Direction::Down);
            });
            model_mesh.up.iter().for_each(|face| {
                add_face(face, Direction::Up);
            });
            model_mesh.north.iter().for_each(|face| {
                add_face(face, Direction::North);
            });
            model_mesh.south.iter().for_each(|face| {
                add_face(face, Direction::South);
            });
            model_mesh.any.iter().for_each(|face| {
                let light_level: LightLevel = state_provider.get_light_level(pos);
                add_quad(face, light_level, Direction::Up);
            });
        }
    }
    layers
}
