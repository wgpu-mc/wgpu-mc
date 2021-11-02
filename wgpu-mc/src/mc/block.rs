use crate::mc::datapack::{BlockModelData, FaceTexture, NamespacedId};
use crate::mc::resource::ResourceProvider;
use crate::mc::{TextureManager, ATLAS_DIMENSIONS};
use crate::model::ModelVertex;
use crate::texture::UV;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

pub struct BlockModelFaces {
    pub north: [ModelVertex; 6],
    pub east: [ModelVertex; 6],
    pub south: [ModelVertex; 6],
    pub west: [ModelVertex; 6],
    pub up: [ModelVertex; 6],
    pub down: [ModelVertex; 6],
}

// #[derive(Debug)]
pub enum BlockModel {
    Cube(BlockModelFaces),
    Custom(Vec<BlockModelFaces>),
}

pub struct StaticBlock {
    //Not a BlockEntity
    pub name: NamespacedId,
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
        resolved_namespaces: &HashMap<&String, &NamespacedId>,
        tex_manager: &TextureManager
    ) -> Option<[[f32; 2]; 2]> {
        match face {
            None => Some([[0.0, 0.0], [0.0, 0.0]]),
            Some(tex) => {
                let loc = match &tex.texture {
                    NamespacedId::Tag(t) => {
                        let resolved = resolved_namespaces.get(t)?;

                        tex_manager.get(resolved)?
                    }
                    NamespacedId::Resource(res) => tex_manager.get(&tex.texture)?,
                    NamespacedId::Invalid => panic!(),
                };

                const ATLAS: f32 = ATLAS_DIMENSIONS as f32;

                let arr = [
                    [
                        // (loc.0.x + tex.uv.0.x) / ATLAS,
                        // (loc.0.y + tex.uv.0.y) / ATLAS,
                        0.0,
                        0.0
                    ],
                    [
                        // (loc.0.x + tex.uv.1.x) / ATLAS,
                        // (loc.0.y + tex.uv.1.y) / ATLAS,
                        16.0 / ATLAS,
                        16.0 / ATLAS
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
                    NamespacedId::Tag(tag) => {
                        let mut value = textures_ids.get(tag)?;

                        while value.is_tag() {
                            if let NamespacedId::Tag(tag2) = value {
                                value = textures_ids.get(tag2)?
                            }
                        }

                        (string, value)
                    }
                    NamespacedId::Resource(_) => (string, namespaced),
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
                let uv = tex_manager.get(value).unwrap_or_else(|| {
                    panic!(
                        "\nModel Value Flattened: {:?}\nModel Key {}\nModel {:?}",
                        value, key, model.id
                    )
                });
                textures.insert((*key).clone(), *uv);
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

                //let resolved_texture_namespaces = ();

                //to-from to model coords is Z and Y inverted

                // let a = [element.from.0, 1.0-element.to.1, 1.0-element.to.2];
                // let b = [element.to.0, 1.0-element.to.1, 1.0-element.to.2];
                // let c = [element.to.0, 1.0-element.from.1, 1.0-element.to.2];
                // let d = [element.from.0, 1.0-element.from.1, 1.0-element.to.2];
                // let e = [element.from.0, 1.0-element.to.1, 1.0-element.from.2];
                // let f = [element.to.0, 1.0-element.to.1, 1.0-element.from.2];
                // let g = [element.to.0, 1.0-element.from.1, 1.0-element.from.2];
                // let h = [element.from.0, 1.0-element.from.1, 1.0-element.from.2];
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
                        ModelVertex { position: e, tex_coords: [south[1][0], south[1][1]], normal: [0.0, 0.0, -1.0], },
                        ModelVertex { position: h, tex_coords: [south[1][0], south[0][1]], normal: [0.0, 0.0, -1.0], },
                        ModelVertex { position: f, tex_coords: [south[0][0], south[1][1]], normal: [0.0, 0.0, -1.0], },
                        ModelVertex { position: h, tex_coords: [south[1][0], south[0][1]], normal: [0.0, 0.0, -1.0], },
                        ModelVertex { position: g, tex_coords: [south[0][0], south[0][1]], normal: [0.0, 0.0, -1.0], },
                        ModelVertex { position: f, tex_coords: [south[0][0], south[1][1]], normal: [0.0, 0.0, -1.0], },
                    ],
                    west: [
                        ModelVertex { position: g, tex_coords: [west[1][0], west[0][1]], normal: [-1.0, 0.0, 0.0], },
                        ModelVertex { position: b, tex_coords: [west[0][0], west[1][1]], normal: [-1.0, 0.0, 0.0], },
                        ModelVertex { position: f, tex_coords: [west[1][0], west[1][1]], normal: [-1.0, 0.0, 0.0], },
                        ModelVertex { position: c, tex_coords: [west[0][0], west[0][1]], normal: [-1.0, 0.0, 0.0], },
                        ModelVertex { position: b, tex_coords: [west[0][0], west[1][1]], normal: [-1.0, 0.0, 0.0], },
                        ModelVertex { position: g, tex_coords: [west[1][0], west[0][1]], normal: [-1.0, 0.0, 0.0], },
                    ], north: [
                        ModelVertex { position: c, tex_coords: [north[1][0], north[0][1]], normal: [0.0, 0.0, 1.0], },
                        ModelVertex { position: a, tex_coords: [north[0][0], north[1][1]], normal: [0.0, 0.0, 1.0], },
                        ModelVertex { position: b, tex_coords: [north[1][0], north[1][1]], normal: [0.0, 0.0, 1.0], },
                        ModelVertex { position: d, tex_coords: [north[0][0], north[0][1]], normal: [0.0, 0.0, 1.0], },
                        ModelVertex { position: a, tex_coords: [north[0][0], north[1][1]], normal: [0.0, 0.0, 1.0], },
                        ModelVertex { position: c, tex_coords: [north[1][0], north[0][1]], normal: [0.0, 0.0, 1.0], },
                    ],
                    east: [
                        ModelVertex { position: e, tex_coords: [east[0][0], east[1][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: a, tex_coords: [east[1][0], east[1][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: d, tex_coords: [east[1][0], east[0][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: d, tex_coords: [east[1][0], east[0][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: h, tex_coords: [east[0][0], east[0][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: e, tex_coords: [east[0][0], east[1][1]], normal: [1.0, 0.0, 0.0], },
                    ],
                    up: [
                        ModelVertex { position: g, tex_coords: [up[1][0], up[0][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: h, tex_coords: [up[0][0], up[0][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: d, tex_coords: [up[0][0], up[1][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: c, tex_coords: [up[1][0], up[1][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: g, tex_coords: [up[1][0], up[0][1]], normal: [1.0, 0.0, 0.0], },
                        ModelVertex { position: d, tex_coords: [up[0][0], up[1][1]], normal: [1.0, 0.0, 0.0], },
                    ],
                    down: [
                        ModelVertex { position: f, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0], },
                        ModelVertex { position: b, tex_coords: [down[0][0], down[0][1]], normal: [0.0, -1.0, 0.0], },
                        ModelVertex { position: a, tex_coords: [down[1][0], down[0][1]], normal: [0.0, -1.0, 0.0], },
                        ModelVertex { position: f, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0], },
                        ModelVertex { position: a, tex_coords: [down[1][0], down[0][1]], normal: [0.0, -1.0, 0.0], },
                        ModelVertex { position: e, tex_coords: [down[1][0], down[1][1]], normal: [0.0, -1.0, 0.0], },
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
            model: if is_cube {
                BlockModel::Cube(results.pop().unwrap().unwrap())
            } else {
                BlockModel::Custom(results.into_iter().map(|x| x.unwrap()).collect())
            },
        })
    }
}

impl Block for StaticBlock {
    fn get_id(&self) -> &NamespacedId {
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
    fn get_id(&self) -> &NamespacedId;
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
    fn get_id(&self) -> &NamespacedId {
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
