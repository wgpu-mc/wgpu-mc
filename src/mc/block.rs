use crate::model::{Vertex, ModelVertex};
use crate::texture::{TextureId, Texture, UV};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use wgpu::{VertexBufferDescriptor, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupEntry};
use crate::mc::datapack::{BlockModelData, NamespacedId, FaceTexture};
use crate::mc::resource::ResourceProvider;
use crate::mc::{TextureManager, ATLAS_DIMENSIONS};

pub struct BlockModelFaces {
    pub north: [ModelVertex; 6],
    pub east: [ModelVertex; 6],
    pub south: [ModelVertex; 6],
    pub west: [ModelVertex; 6],
    pub up: [ModelVertex; 6],
    pub down: [ModelVertex; 6]
}

// #[derive(Debug)]
pub enum BlockModel {
    Cube(BlockModelFaces),
    Custom(Vec<BlockModelFaces>)
}

pub struct StaticBlock { //Not a BlockEntity
    pub name: NamespacedId,
    pub textures: HashMap<String, UV>,
    pub model: BlockModel
}

macro_rules! upload_vertex_vec {
    ($device:ident, $vec:expr) => {
        $device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&$vec[..]),
            usage: wgpu::BufferUsage::VERTEX
        })
    }
}

impl StaticBlock {

    pub fn get_element_face_uv(face: &Option<FaceTexture>, resolved_namespaces: &HashMap<String, NamespacedId>, tex_manager: &TextureManager, name: &str) -> Option<[[f32; 2]; 2]> {
        Option::Some(match face {
            None => [[0.0, 0.0], [0.0, 0.0]],
            Some(tex) => {
                let loc = match &tex.texture {
                    NamespacedId::Tag(t) => {
                        let resolved  = resolved_namespaces.get(t)?;

                        tex_manager.get( resolved )?
                    },
                    NamespacedId::Resource(res) => tex_manager.get( &tex.texture )?,
                    NamespacedId::Invalid => panic!()
                };

                // if name == "minecraft:block/anvil" {
                //     println!("loc.0.x + tex.uv.0.x = {} + {} = {}", loc.0.x, tex.uv.0.x, loc.0.x + tex.uv.0.x);
                //     println!("loc.0.x + tex.uv.0.y = {} + {} = {}", loc.0.x, tex.uv.0.y, loc.0.x + tex.uv.0.y);
                //     println!("loc.0.x + tex.uv.1.x = {} + {} = {}", loc.0.x, tex.uv.1.x, loc.0.x + tex.uv.1.x);
                //     println!("loc.0.x + tex.uv.1.y = {} + {} = {}", loc.0.x, tex.uv.1.y, loc.0.x + tex.uv.1.x);
                // }
                let atlas = ATLAS_DIMENSIONS as f32;

                let arr = [
                    [
                        (loc.0.x + tex.uv.0.x) / atlas,
                        (loc.0.y + tex.uv.0.y) / atlas
                        // 0.0, 0.0
                    ],
                    [
                        (loc.0.x + tex.uv.1.x) / atlas,
                        (loc.0.y + tex.uv.1.y) / atlas
                        // 0.015625, 0.015625
                    ]
                ];

                arr
            }
        })
    }

    pub fn from_datapack(device: &wgpu::Device, model: &BlockModelData, rp: &dyn ResourceProvider, tex_manager: &TextureManager)-> Option<Self> {
        let textures_ids = model.textures.clone();

        let mut textures = HashMap::new();

        let resolved_texture_namespaces_vec = textures_ids.iter().map(|(string, namespaced)| {
            Option::Some(match namespaced {
                NamespacedId::Tag(tag) => {
                    let mut value = textures_ids.get(tag)?;

                    while value.is_tag() {
                        match value {
                            NamespacedId::Tag(tag2) => value = textures_ids.get(tag2)?,
                            _ => unreachable!()
                        }
                    }

                    (string.clone(), value.clone())
                },
                NamespacedId::Resource(_) => {
                    (string.clone(), namespaced.clone())
                }
                _ => panic!()
            })
        }).collect::<Vec<Option<(String, NamespacedId)>>>();

        for i in resolved_texture_namespaces_vec.iter() {
            if i.is_none() {
                return Option::None;
            }
        }

        let resolved_texture_namespaces = resolved_texture_namespaces_vec.into_iter().map(|x| x.unwrap())
            .collect::<HashMap<String, NamespacedId>>();

        resolved_texture_namespaces.iter().for_each(|(key, value)| {
            let uv = tex_manager.get(value).expect(&format!("\nModel Value Flattened: {:?}\nModel Key {}\nModel {:?}", value, key, model.id));
            textures.insert(key.clone(), uv.clone());
        }); //Map the referenced textures to their respective UVs in the texture atlas

        let is_cube = if model.elements.len() == 1 {
            let first = model.elements.first().unwrap();

            first.from.0 == 0.0 && first.from.1 == 1.0 && first.from.2 == 1.0 &&
                first.to.0 == 1.0 && first.to.1 == 0.0 && first.to.2 == 0.0
        } else {
            false
        };

        let mut results = model.elements.iter().map(|element| { //TODO: properly generate the vertices, probably in another method
            // if model.id == NamespacedId::from("minecraft:block/cobblestone") {
            //     println!("To {:?}\nFrom {:?}", element.from, element.to);
            // }

            let name = &model.id.to_str();

            //Face textures
            let north = Self::get_element_face_uv(&element.face_textures.north, &resolved_texture_namespaces, tex_manager, name)?;
            let east = Self::get_element_face_uv(&element.face_textures.east, &resolved_texture_namespaces, tex_manager, name)?;
            let south = Self::get_element_face_uv(&element.face_textures.south, &resolved_texture_namespaces, tex_manager, name)?;
            let west = Self::get_element_face_uv(&element.face_textures.west, &resolved_texture_namespaces, tex_manager, name)?;
            let down = Self::get_element_face_uv(&element.face_textures.down, &resolved_texture_namespaces, tex_manager, name)?;
            let up = Self::get_element_face_uv(&element.face_textures.up, &resolved_texture_namespaces, tex_manager, name)?;

            // let a = [element.from.0, element.from.1, element.from.2];
            // let b = [element.to.0, element.from.1, element.from.2];
            // let c = [element.to.0, element.to.1, element.from.2];
            // let d = [element.from.0, element.to.1, element.from.2];
            // let e = [element.from.0, element.from.1, element.to.2];
            // let f = [element.to.0, element.from.1, element.to.2];
            // let g = [element.to.0, element.to.1, element.to.2];
            // let h = [element.from.0, element.to.1, element.to.2];
            let a = [element.to.0, element.from.1, element.from.2];
            let b = [element.from.0, element.from.1, element.from.2];
            let c = [element.from.0, element.to.1, element.from.2];
            let d = [element.to.0, element.to.1, element.from.2];
            let e = [element.to.0, element.from.1, element.to.2];
            let f = [element.from.0, element.from.1, element.to.2];
            let g = [element.from.0, element.to.1, element.to.2];
            let h = [element.to.0, element.to.1, element.to.2];

            let faces = BlockModelFaces {
                south: [
                    ModelVertex { position: c, tex_coords: [south[1][0], south[0][1]], normal: [0.0, 0.0, 1.0] },
                    ModelVertex { position: a, tex_coords: [south[0][0], south[1][1]], normal: [0.0, 0.0, 1.0] },
                    ModelVertex { position: b, tex_coords: [south[1][0], south[1][1]], normal: [0.0, 0.0, 1.0] },
                    ModelVertex { position: d, tex_coords: [south[0][0], south[0][1]], normal: [0.0, 0.0, 1.0] },
                    ModelVertex { position: a, tex_coords: [south[0][0], south[1][1]], normal: [0.0, 0.0, 1.0] },
                    ModelVertex { position: c, tex_coords: [south[1][0], south[0][1]], normal: [0.0, 0.0, 1.0] }
                ],
                west: [
                    ModelVertex { position: g, tex_coords: [west[1][0], west[0][1]], normal: [-1.0, 0.0, 0.0] },
                    ModelVertex { position: b, tex_coords: [west[0][0], west[1][1]], normal: [-1.0, 0.0, 0.0] },
                    ModelVertex { position: f, tex_coords: [west[1][0], west[1][1]], normal: [-1.0, 0.0, 0.0] },
                    ModelVertex { position: c, tex_coords: [west[0][0], west[0][1]], normal: [-1.0, 0.0, 0.0] },
                    ModelVertex { position: b, tex_coords: [west[0][0], west[1][1]], normal: [-1.0, 0.0, 0.0] },
                    ModelVertex { position: g, tex_coords: [west[1][0], west[0][1]], normal: [-1.0, 0.0, 0.0] }
                ],
                north: [
                    ModelVertex { position: f, tex_coords: [north[0][0], north[1][1]], normal: [0.0, 0.0, -1.0] },
                    ModelVertex { position: e, tex_coords: [north[1][0], north[1][1]], normal: [0.0, 0.0, -1.0] },
                    ModelVertex { position: h, tex_coords: [north[1][0], north[0][1]], normal: [0.0, 0.0, -1.0] },
                    ModelVertex { position: h, tex_coords: [north[1][0], north[0][1]], normal: [0.0, 0.0, -1.0] },
                    ModelVertex { position: g, tex_coords: [north[0][0], north[0][1]], normal: [0.0, 0.0, -1.0] },
                    ModelVertex { position: f, tex_coords: [north[0][0], north[1][1]], normal: [0.0, 0.0, -1.0] }
                ],
                east: [
                    ModelVertex { position: a, tex_coords: [east[1][0], east[1][1]], normal: [1.0, 0.0, 0.0] },
                    ModelVertex { position: d, tex_coords: [east[1][0], east[0][1]], normal: [1.0, 0.0, 0.0] },
                    ModelVertex { position: e, tex_coords: [east[0][0], east[1][1]], normal: [1.0, 0.0, 0.0] },
                    ModelVertex { position: d, tex_coords: [east[1][0], east[0][1]], normal: [1.0, 0.0, 0.0] },
                    ModelVertex { position: h, tex_coords: [east[0][0], east[0][1]], normal: [1.0, 0.0, 0.0] },
                    ModelVertex { position: e, tex_coords: [east[0][0], east[1][1]], normal: [1.0, 0.0, 0.0] }
                ],
                up: [
                    ModelVertex { position: f, tex_coords: [down[0][0], down[1][1]], normal: [0.0, 1.0, 0.0] },
                    ModelVertex { position: b, tex_coords: [down[0][0], down[0][1]], normal: [0.0, 1.0, 0.0] },
                    ModelVertex { position: a, tex_coords: [down[1][0], down[0][1]], normal: [0.0, 1.0, 0.0] },
                    ModelVertex { position: f, tex_coords: [down[0][0], down[1][1]], normal: [0.0, 1.0, 0.0] },
                    ModelVertex { position: a, tex_coords: [down[1][0], down[0][1]], normal: [0.0, 1.0, 0.0] },
                    ModelVertex { position: e, tex_coords: [down[1][0], down[1][1]], normal: [0.0, 1.0, 0.0] }
                ],
                down: [
                    ModelVertex { position: d, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0] },
                    ModelVertex { position: g, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0] },
                    ModelVertex { position: c, tex_coords: [down[1][0], down[1][1]], normal: [0.0, -1.0, 0.0] },
                    ModelVertex { position: h, tex_coords: [down[0][0], down[0][1]], normal: [0.0, -1.0, 0.0] },
                    ModelVertex { position: g, tex_coords: [down[0][0], down[1][1]], normal: [0.0, -1.0, 0.0] },
                    ModelVertex { position: d, tex_coords: [down[0][0], down[0][1]], normal: [0.0, -1.0, 0.0] }
                ]
            };

            Option::Some(faces)
        }).collect::<Vec<Option<BlockModelFaces>>>();

        for e in results.iter() {
            if e.is_none() {
                return None;
            }
        }

        Option::Some(Self {
            name: model.id.clone(),
            textures,
            model: if is_cube {
                BlockModel::Cube(results.pop().unwrap().unwrap())
            } else {
                BlockModel::Custom(results.into_iter().map(|x| x.unwrap()).collect())
            }
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
    Down
}

pub enum BlockEntityDataKey {
    ChestOpenTime
}

pub struct BlockEntity<'block> {
    pub block: &'block dyn Block,
    pub data: HashMap<BlockEntityDataKey, usize>
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
    pub is_cube: bool //speed things up a bit
}