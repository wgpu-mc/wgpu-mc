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
use std::ops::Range;
use std::sync::Arc;

use glam::{ivec3, IVec2, IVec3, Vec2Swizzles, Vec3Swizzles};
use range_alloc::RangeAllocator;

use crate::mc::block::{BlockstateKey, ChunkBlockState, ModelMesh};
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
    fn get_state(&self, x: i32, y: i32, z: i32) -> ChunkBlockState;

    fn get_light_level(&self, x: i32, y: i32, z: i32) -> LightLevel;

    fn is_section_empty(&self, rel_pos: IVec3) -> bool;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum RenderLayer {
    Solid=0,
    Cutout=1,
    Transparent=2,
}

#[derive(Clone)]
pub struct SectionRanges {
    pub vertex_range: Range<u32>,
    pub index_range: Range<u32>,
}


///The struct representing a Chunk section, with various render layers, split into sections
pub struct SectionStorage{
    storage:HashMap<IVec3,Section>,
    allocator:RangeAllocator<u32>,
    width:i32,
}
impl SectionStorage {
    pub fn new(range:u32)->Self{
        SectionStorage{
            storage:HashMap::new(),
            width:0,
            allocator: RangeAllocator::new(0..range),
        }
    }
    pub fn clear(&mut self){
        self.allocator.reset();
        self.storage.clear();
    }
    pub fn set_width(&mut self,w:i32){
        self.width = w;
    }
    pub fn trim(&mut self,pos:IVec2){
        let mut to_remove = vec![];
        for (k,section) in &self.storage{
            let dist = (k.xz()-pos).abs();
            let radius = self.width + 2;//temp fix until proper sync
            if dist.x>radius || dist.y>radius{
                to_remove.push(*k);
                for layer in &section.layers{
                    if let Some(l) = layer.as_ref(){
                        self.allocator.free_range(l.vertex_range.clone());
                        self.allocator.free_range(l.index_range.clone());
                    }
                }
            }
        }
        to_remove.iter().for_each(|pos|{self.storage.remove(pos);});
    }
    pub fn replace(&mut self, pos:IVec3,baked_layers:&Vec<BakedLayer>)->Section{
        if let Some(previous_section) = self.storage.get(&pos){
            for layer in &previous_section.layers{
                if let Some(l) = layer.as_ref(){
                    self.allocator.free_range(l.vertex_range.clone());
                    self.allocator.free_range(l.index_range.clone());
                }
            }
        }
        let section = Section{layers:baked_layers.iter().map(|layer|{
            if layer.indices.len()>0{
                Some(SectionRanges{
                    vertex_range:self.allocator.allocate_range(layer.vertices.len() as u32/4).unwrap(),
                    index_range:self.allocator.allocate_range(layer.indices.len() as u32/4).unwrap()
                })
            }
            else{
                None
            }
        }).collect()};
        self.storage.insert(pos,section.clone());
        section
    }
    pub fn iter(&self)->std::collections::hash_map::Iter<IVec3, Section>{
        self.storage.iter()
    }
}

#[derive(Clone)]
pub struct Section {
    pub layers: Vec<Option<SectionRanges>>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
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

pub fn bake_section<Provider: BlockStateProvider>(pos: IVec3, wm:&WmRenderer ,bsp: &Provider, ) {

    let bm = wm.mc.block_manager.read();

    let baked_section = bake_layers(pos, &bm, bsp);

    wm.chunk_update_queue.0.send((pos,baked_section)).unwrap();
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
    let mut layers = vec![BakedLayer::default();3];

    if state_provider.is_section_empty(ivec3(0, 0, 0)) {
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
                        let baked_layer = &mut layers[layer as usize];
                        let vec_index = baked_layer.vertices.len()/Vertex::VERTEX_LENGTH;

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
                            }).flat_map(Vertex::compressed));
                        baked_layer.indices.extend(INDICES.iter().flat_map(|index| (index + (vec_index as u32)).to_ne_bytes()));
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
                    let baked_layer = &mut layers[model_mesh.layer as usize];
                    let vec_index = baked_layer.vertices.len()/Vertex::VERTEX_LENGTH;

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
                        }).flat_map(Vertex::compressed));
                    baked_layer.indices.extend(INDICES.iter().flat_map(|index| (index + (vec_index as u32)).to_ne_bytes()));
                });
            }
        }
    }

    layers
}
