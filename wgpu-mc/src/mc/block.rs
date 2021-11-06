use crate::mc::datapack::{BlockModelData, FaceTexture, Identifier};
use crate::mc::resource::ResourceProvider;
use crate::model::MeshVertex;
use crate::texture::UV;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use crate::render::atlas::{TextureManager, ATLAS_DIMENSIONS};

pub struct BlockModelFaces {
    pub north: [MeshVertex; 6],
    pub east: [MeshVertex; 6],
    pub south: [MeshVertex; 6],
    pub west: [MeshVertex; 6],
    pub up: [MeshVertex; 6],
    pub down: [MeshVertex; 6],
}

// #[derive(Debug)]
pub enum BlockModel {
    Cube(BlockModelFaces),
    Custom(Vec<BlockModelFaces>),
}

pub struct StaticBlock {
    //Not a BlockEntity
    pub name: Identifier,
    pub textures: HashMap<String, UV>,
    pub model: BlockModel,
}

#[allow(unused_macros)] // TODO
macro_rules! upload_vertex_vec {
    ($device:ident, $vec:expr) => {
        $device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&$vec[..]),
            usage: wgpu::BufferUsage::VERTEX,
        })
    };
}

impl StaticBlock {

    pub fn get_element_face_uv(
        face: &Option<FaceTexture>,
        resolved_identifiers: &HashMap<&String, &Identifier>,
        tex_manager: &TextureManager
    ) -> Option<[[f32; 2]; 2]> {
        match face {
            None => Some([[0.0, 0.0], [0.0, 0.0]]),
            Some(tex) => {
                let loc = match &tex.texture {
                    Identifier::Tag(t) => {
                        let resolved = resolved_identifiers.get(t)?;

                        tex_manager.atlases.block.map.as_ref()?.get(resolved)?
                    }
                    Identifier::Resource(res) => tex_manager.atlases.block.map.as_ref()?.get(&tex.texture)?
                };

                const ATLAS: f32 = ATLAS_DIMENSIONS as f32;

                let arr = [
                    [
                        (loc.0.x + tex.uv.0.x) / ATLAS,
                        (loc.0.y + tex.uv.0.y) / ATLAS
                    ],
                    [
                        (loc.0.x + tex.uv.1.x) / ATLAS,
                        (loc.0.y + tex.uv.1.y) / ATLAS
                    ],
                ];

                Some(arr)
            }
        }
    }

    #[allow(unused_variables)] // TODO parameters device and rp are unused
    pub fn from_datapack(
        device: &wgpu::Device,
        model: &BlockModelData,
        rp: &dyn ResourceProvider,
        tex_manager: &TextureManager,
    ) -> Option<Self> {
        let textures_ids = &model.textures;

        let resolved_texture_namespaces_vec: Vec<_> = textures_ids
            .iter()
            .map(|(string, namespaced)| {
                Some(match namespaced {
                    Identifier::Tag(tag) => {
                        let mut value = textures_ids.get(tag)?;

                        while value.is_tag() {
                            if let Identifier::Tag(tag2) = value {
                                value = textures_ids.get(tag2)?
                            }
                        }

                        (string, value)
                    }
                    Identifier::Resource(_) => (string, namespaced),
                    _ => panic!(),
                })
            })
            .collect();

        if resolved_texture_namespaces_vec.iter().any(|x| x.is_none()) {
            return None;
        }

        let resolved_texture_namespaces: HashMap<_, _> = resolved_texture_namespaces_vec
            .into_iter()
            .map(|x| x.unwrap())
            .collect();

        let textures = {
            let mut textures = HashMap::new();

            for (&key, value) in resolved_texture_namespaces.iter() {
                let uv = tex_manager.atlases.block.map.as_ref()?.get(value).unwrap();

                textures.insert(key.clone(), uv.clone());
            } //Map the referenced textures to their respective UVs in the texture atlas
            textures
        };

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
                //TODO: properly generate the vertices, probably in another method
                // if model.id == NamespacedId::from("minecraft:block/cobblestone") {
                //     println!("To {:?}\nFrom {:?}", element.from, element.to);
                // }

                let name = model.id.to_string();

                //Face textures
                let north = Self::get_element_face_uv(
                    &element.face_textures.north,
                    &resolved_texture_namespaces,
                    tex_manager
                )?;
                let east = Self::get_element_face_uv(
                    &element.face_textures.east,
                    &resolved_texture_namespaces,
                    tex_manager
                )?;
                let south = Self::get_element_face_uv(
                    &element.face_textures.south,
                    &resolved_texture_namespaces,
                    tex_manager
                )?;
                let west = Self::get_element_face_uv(
                    &element.face_textures.west,
                    &resolved_texture_namespaces,
                    tex_manager
                )?;
                let down = Self::get_element_face_uv(
                    &element.face_textures.down,
                    &resolved_texture_namespaces,
                    tex_manager
                )?;
                let up = Self::get_element_face_uv(
                    &element.face_textures.up,
                    &resolved_texture_namespaces,
                    tex_manager
                )?;

                dbg!(north, east, south, west, down, up);

                let a = [1.0-element.from.0, element.from.1,   element.from.2];
                let b = [1.0-element.to.0, element.from.1,     element.from.2];
                let c = [1.0-element.to.0, element.to.1,   element.from.2];
                let d = [1.0-element.from.0, element.to.1, element.from.2];
                let e = [1.0-element.from.0, element.from.1,   element.to.2];
                let f = [1.0-element.to.0, element.from.1,     element.to.2];
                let g = [1.0-element.to.0, element.to.1,   element.to.2];
                let h = [1.0-element.from.0, element.to.1, element.to.2];

                #[rustfmt::skip]
                let faces = BlockModelFaces {
                    south: [
                        MeshVertex { position: e, tex_coords: [south[1][0], south[1][1]], normal: [0.0, 0.0, -1.0], },
                        MeshVertex { position: h, tex_coords: [south[1][0], south[0][1]], normal: [0.0, 0.0, -1.0], },
                        MeshVertex { position: f, tex_coords: [south[0][0], south[1][1]], normal: [0.0, 0.0, -1.0], },
                        MeshVertex { position: h, tex_coords: [south[1][0], south[0][1]], normal: [0.0, 0.0, -1.0], },
                        MeshVertex { position: g, tex_coords: [south[0][0], south[0][1]], normal: [0.0, 0.0, -1.0], },
                        MeshVertex { position: f, tex_coords: [south[0][0], south[1][1]], normal: [0.0, 0.0, -1.0], },
                    ],
                    west: [
                        MeshVertex { position: g, tex_coords: [west[1][0], west[0][1]], normal: [-1.0, 0.0, 0.0], },
                        MeshVertex { position: b, tex_coords: [west[0][0], west[1][1]], normal: [-1.0, 0.0, 0.0], },
                        MeshVertex { position: f, tex_coords: [west[1][0], west[1][1]], normal: [-1.0, 0.0, 0.0], },
                        MeshVertex { position: c, tex_coords: [west[0][0], west[0][1]], normal: [-1.0, 0.0, 0.0], },
                        MeshVertex { position: b, tex_coords: [west[0][0], west[1][1]], normal: [-1.0, 0.0, 0.0], },
                        MeshVertex { position: g, tex_coords: [west[1][0], west[0][1]], normal: [-1.0, 0.0, 0.0], },
                    ], north: [
                        MeshVertex { position: c, tex_coords: [north[1][0], north[0][1]], normal: [0.0, 0.0, 1.0], },
                        MeshVertex { position: a, tex_coords: [north[0][0], north[1][1]], normal: [0.0, 0.0, 1.0], },
                        MeshVertex { position: b, tex_coords: [north[1][0], north[1][1]], normal: [0.0, 0.0, 1.0], },
                        MeshVertex { position: d, tex_coords: [north[0][0], north[0][1]], normal: [0.0, 0.0, 1.0], },
                        MeshVertex { position: a, tex_coords: [north[0][0], north[1][1]], normal: [0.0, 0.0, 1.0], },
                        MeshVertex { position: c, tex_coords: [north[1][0], north[0][1]], normal: [0.0, 0.0, 1.0], },
                    ],
                    east: [
                        MeshVertex { position: e, tex_coords: [east[0][0], east[1][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: a, tex_coords: [east[1][0], east[1][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: d, tex_coords: [east[1][0], east[0][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: d, tex_coords: [east[1][0], east[0][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: h, tex_coords: [east[0][0], east[0][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: e, tex_coords: [east[0][0], east[1][1]], normal: [1.0, 0.0, 0.0], },
                    ],
                    up: [
                        MeshVertex { position: g, tex_coords: [up[1][0], up[0][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: h, tex_coords: [up[0][0], up[0][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: d, tex_coords: [up[0][0], up[1][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: c, tex_coords: [up[1][0], up[1][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: g, tex_coords: [up[1][0], up[0][1]], normal: [1.0, 0.0, 0.0], },
                        MeshVertex { position: d, tex_coords: [up[0][0], up[1][1]], normal: [1.0, 0.0, 0.0], },
                    ],
                    down: [
                        MeshVertex { position: f, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0], },
                        MeshVertex { position: b, tex_coords: [down[0][0], down[0][1]], normal: [0.0, -1.0, 0.0], },
                        MeshVertex { position: a, tex_coords: [down[1][0], down[0][1]], normal: [0.0, -1.0, 0.0], },
                        MeshVertex { position: f, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0], },
                        MeshVertex { position: a, tex_coords: [down[1][0], down[0][1]], normal: [0.0, -1.0, 0.0], },
                        MeshVertex { position: e, tex_coords: [down[1][0], down[1][1]], normal: [0.0, -1.0, 0.0], },
                    ],
                };

                dbg!(&faces.north);

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
            model: if is_cube {
                BlockModel::Cube(results.pop().unwrap().unwrap())
            } else {
                BlockModel::Custom(results.into_iter().map(|x| x.unwrap()).collect())
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

    fn get_model(&self) -> &BlockModel {
        &self.model
    }
}

pub trait Block {
    fn get_id(&self) -> &Identifier;
    fn get_textures(&self) -> &HashMap<String, UV>;
    fn get_model(&self) -> &BlockModel;
}

#[derive(Clone, Copy)]
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

    fn get_model(&self) -> &BlockModel {
        self.block.get_model()
    }
}

pub type BlockPos = (u32, u8, u32);

type BlockIndex = usize;

#[derive(Clone, Copy)]
pub struct BlockState {
    pub block: Option<BlockIndex>,
    pub direction: BlockDirection,
    pub damage: u8,
    pub transparency: bool, //speed things up a bit
}
