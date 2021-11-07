use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::Arc;

use crate::mc::resource::ResourceProvider;
use crate::model::MeshVertex;
use crate::render::atlas::{ATLAS_DIMENSIONS, TextureManager};
use crate::texture::UV;
use crate::mc::datapack::{Identifier, FaceTexture, BlockModel};

#[derive(Debug)]
pub struct BlockModelFaces {
    pub north: [MeshVertex; 6],
    pub east: [MeshVertex; 6],
    pub south: [MeshVertex; 6],
    pub west: [MeshVertex; 6],
    pub up: [MeshVertex; 6],
    pub down: [MeshVertex; 6],
}

#[derive(Debug)]
pub enum BlockShape {
    Cube(BlockModelFaces),
    Custom(Vec<BlockModelFaces>),
}

///Non-block entity block
pub struct StaticBlock {
    pub name: Identifier,
    pub textures: HashMap<String, UV>,
    pub shape: BlockShape,
}

impl StaticBlock {
    pub fn relative_atlas_uv(
        face: &Option<FaceTexture>,
        textures: &HashMap<String, Identifier>,
        tex_manager: &TextureManager,
    ) -> Option<UV> {
        let atlas_uv = face.as_ref().map_or(((0.0, 0.0), (0.0, 0.0)), |texture| {
            let atlases = tex_manager.atlases.read();
            atlases.block.map.get(&texture.texture).unwrap().clone()
        });

        let face_uv = face.as_ref().map_or(((0.0, 0.0), (0.0, 0.0)), |texture| {
            texture.uv
        });

        const ATLAS: f32 = ATLAS_DIMENSIONS as f32;

        let adjusted_uv = (
            (
                (atlas_uv.0.0 + face_uv.0.0) / ATLAS,
                (atlas_uv.0.1 + face_uv.0.1) / ATLAS
            ),
            (
                (atlas_uv.1.0 + face_uv.1.0) / ATLAS,
                (atlas_uv.1.1 + face_uv.1.1) / ATLAS
            )
        );

        Some(adjusted_uv)
    }

    #[allow(unused_variables)] // TODO parameters device and rp are unused
    pub fn from_datapack(
        device: &wgpu::Device,
        model: &BlockModel,
        rp: &dyn ResourceProvider,
        tex_manager: &TextureManager,
    ) -> Option<Self> {
        let texture_ids = &model.textures;

        let textures: HashMap<String, UV> = texture_ids.iter().map(|(key, identifier)| {
            (key.clone(), tex_manager.atlases.read().block.map.get(identifier).unwrap().clone())
        }).collect();

        let is_cube = model.elements.len() == 1 && {
            let first = model.elements.first().unwrap();

            first.from.0 == 0.0
                && first.from.1 == 0.0
                && first.from.2 == 0.0
                && first.to.0 == 1.0
                && first.to.1 == 1.0
                && first.to.2 == 1.0
        };

        let mut results = model
            .elements
            .iter()
            .map(|element| {
                let name = model.id.to_string();

                //Face textures
                let north = Self::relative_atlas_uv(
                    &element.face_textures.north,
                    &texture_ids,
                    tex_manager,
                )?;
                let east = Self::relative_atlas_uv(
                    &element.face_textures.east,
                    &texture_ids,
                    tex_manager,
                )?;
                let south = Self::relative_atlas_uv(
                    &element.face_textures.south,
                    &texture_ids,
                    tex_manager,
                )?;
                let west = Self::relative_atlas_uv(
                    &element.face_textures.west,
                    &texture_ids,
                    tex_manager,
                )?;
                let down = Self::relative_atlas_uv(
                    &element.face_textures.down,
                    &texture_ids,
                    tex_manager,
                )?;
                let up = Self::relative_atlas_uv(
                    &element.face_textures.up,
                    &texture_ids,
                    tex_manager,
                )?;

                let a = [1.0 - element.from.0, element.from.1, element.from.2];
                let b = [1.0 - element.to.0, element.from.1, element.from.2];
                let c = [1.0 - element.to.0, element.to.1, element.from.2];
                let d = [1.0 - element.from.0, element.to.1, element.from.2];
                let e = [1.0 - element.from.0, element.from.1, element.to.2];
                let f = [1.0 - element.to.0, element.from.1, element.to.2];
                let g = [1.0 - element.to.0, element.to.1, element.to.2];
                let h = [1.0 - element.from.0, element.to.1, element.to.2];

                #[rustfmt::skip]
                    let faces = BlockModelFaces {
                    south: [
                        MeshVertex { position: e, tex_coords: [south.1.0, south.1.1], normal: [0.0, 0.0, -1.0] },
                        MeshVertex { position: h, tex_coords: [south.1.0, south.0.1], normal: [0.0, 0.0, -1.0] },
                        MeshVertex { position: f, tex_coords: [south.0.0, south.1.1], normal: [0.0, 0.0, -1.0] },
                        MeshVertex { position: h, tex_coords: [south.1.0, south.0.1], normal: [0.0, 0.0, -1.0] },
                        MeshVertex { position: g, tex_coords: [south.0.0, south.0.1], normal: [0.0, 0.0, -1.0] },
                        MeshVertex { position: f, tex_coords: [south.0.0, south.1.1], normal: [0.0, 0.0, -1.0] },
                    ],
                    west: [
                        MeshVertex { position: g, tex_coords: [west.1.0, west.0.1], normal: [-1.0, 0.0, 0.0] },
                        MeshVertex { position: b, tex_coords: [west.0.0, west.1.1], normal: [-1.0, 0.0, 0.0] },
                        MeshVertex { position: f, tex_coords: [west.1.0, west.1.1], normal: [-1.0, 0.0, 0.0] },
                        MeshVertex { position: c, tex_coords: [west.0.0, west.0.1], normal: [-1.0, 0.0, 0.0] },
                        MeshVertex { position: b, tex_coords: [west.0.0, west.1.1], normal: [-1.0, 0.0, 0.0] },
                        MeshVertex { position: g, tex_coords: [west.1.0, west.0.1], normal: [-1.0, 0.0, 0.0] },
                    ],
                    north: [
                        MeshVertex { position: c, tex_coords: [north.1.0, north.0.1], normal: [0.0, 0.0, 1.0] },
                        MeshVertex { position: a, tex_coords: [north.0.0, north.1.1], normal: [0.0, 0.0, 1.0] },
                        MeshVertex { position: b, tex_coords: [north.1.0, north.1.1], normal: [0.0, 0.0, 1.0] },
                        MeshVertex { position: d, tex_coords: [north.0.0, north.0.1], normal: [0.0, 0.0, 1.0] },
                        MeshVertex { position: a, tex_coords: [north.0.0, north.1.1], normal: [0.0, 0.0, 1.0] },
                        MeshVertex { position: c, tex_coords: [north.1.0, north.0.1], normal: [0.0, 0.0, 1.0] },
                    ],
                    east: [
                        MeshVertex { position: e, tex_coords: [east.0.0, east.1.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: a, tex_coords: [east.1.0, east.1.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: d, tex_coords: [east.1.0, east.0.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: d, tex_coords: [east.1.0, east.0.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: h, tex_coords: [east.0.0, east.0.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: e, tex_coords: [east.0.0, east.1.1], normal: [1.0, 0.0, 0.0] },
                    ],
                    up: [
                        MeshVertex { position: g, tex_coords: [up.1.0, up.0.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: h, tex_coords: [up.0.0, up.0.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: d, tex_coords: [up.0.0, up.1.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: c, tex_coords: [up.1.0, up.1.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: g, tex_coords: [up.1.0, up.0.1], normal: [1.0, 0.0, 0.0] },
                        MeshVertex { position: d, tex_coords: [up.0.0, up.1.1], normal: [1.0, 0.0, 0.0] },
                    ],
                    down: [
                        MeshVertex { position: f, tex_coords: [down.0.0, down.1.1], normal: [0.0, -1.0, 0.0] },
                        MeshVertex { position: b, tex_coords: [down.0.0, down.0.1], normal: [0.0, -1.0, 0.0] },
                        MeshVertex { position: a, tex_coords: [down.1.0, down.0.1], normal: [0.0, -1.0, 0.0] },
                        MeshVertex { position: f, tex_coords: [down.0.0, down.1.1], normal: [0.0, -1.0, 0.0] },
                        MeshVertex { position: a, tex_coords: [down.1.0, down.0.1], normal: [0.0, -1.0, 0.0] },
                        MeshVertex { position: e, tex_coords: [down.1.0, down.1.1], normal: [0.0, -1.0, 0.0] },
                    ],
                };

                Some(faces)
            })
            .collect::<Vec<Option<BlockModelFaces>>>();

        for e in results.iter() {
            if e.is_none() {
                return None;
            }
        }

        Some(Self {
            name: model.id.clone(),
            textures,
            shape: if is_cube {
                BlockShape::Cube(results.pop().unwrap().unwrap())
            } else {
                BlockShape::Custom(results.into_iter().map(|x| x.unwrap()).collect())
            },
        })
    }
}

impl Block for StaticBlock {
    fn get_id(&self) -> &Identifier {
        &self.name
    }

    fn get_textures(&self) -> &HashMap<String, UV, RandomState> {
        &self.textures
    }

    fn get_shape(&self) -> &BlockShape {
        &self.shape
    }
}

pub trait Block {
    fn get_id(&self) -> &Identifier;
    fn get_textures(&self) -> &HashMap<String, UV>;
    fn get_shape(&self) -> &BlockShape;
}

#[derive(Clone, Copy, Debug)]
pub enum BlockDirection {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

pub enum BlockEntityDataKey {
    ChestOpenTime,
}

pub struct BlockEntity<'block> {
    pub block: &'block dyn Block,
    pub data: HashMap<BlockEntityDataKey, usize>,
}

impl<'block> Block for BlockEntity<'block> {
    fn get_id(&self) -> &Identifier {
        self.block.get_id()
    }

    fn get_textures(&self) -> &HashMap<String, UV> {
        self.block.get_textures()
    }

    fn get_shape(&self) -> &BlockShape {
        self.block.get_shape()
    }
}

pub type BlockPos = (u32, u8, u32);

type BlockIndex = usize;

#[derive(Clone, Copy, Debug)]
pub struct BlockState {
    pub block: Option<BlockIndex>,
    pub direction: BlockDirection,
    pub damage: u8,
    pub transparency: bool, //speed things up a bit
}
