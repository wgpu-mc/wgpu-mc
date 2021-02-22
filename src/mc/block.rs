use crate::model::{Vertex, ModelVertex};
use crate::texture::{TextureId, Texture, UV};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use wgpu::{VertexBufferDescriptor, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupEntry};
use crate::mc::datapack::{BlockModelData, NamespacedId};
use crate::mc::resource::ResourceProvider;
use crate::mc::TextureManager;

#[derive(Debug)]
pub enum BlockModel {
    Cube(wgpu::Buffer, u16),
    Custom(wgpu::Buffer, u16)
}

pub struct StaticBlock { //Not a BlockEntity
    pub name: NamespacedId,
    pub textures: HashMap<String, UV>,
    pub model: BlockModel
}

impl StaticBlock {
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

        let mut vertices = Vec::new();

        let is_cube = if model.elements.len() == 1 {
            let first = model.elements.first().unwrap();

            first.from.0 == 0.0 && first.from.1 == 0.0 && first.from.2 == 0.0 &&
                first.to.0 == 16.0 && first.to.1 == 16.0 && first.to.2 == 16.0
        } else {
            false
        };

        model.elements.iter().for_each(|element| { //TODO: properly generate the vertices, probably in another method
            if model.id == NamespacedId::from("minecraft:block/cobblestone") {
                println!("To {:?}\nFrom {:?}", element.from, element.to);
            }

            let a = [element.from.0, element.from.1, element.from.2];
            let b = [element.to.0, element.from.1, element.from.2];
            let c = [element.to.0, element.to.1, element.from.2];
            let d = [element.from.0, element.to.1, element.from.2];
            let e = [element.from.0, element.from.1, element.to.2];
            let f = [element.to.0, element.from.1, element.to.2];
            let g = [element.to.0, element.to.1, element.to.2];
            let h = [element.from.0, element.to.1, element.to.2];

            vertices.extend(vec![
                //Front
                ModelVertex { position: a, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: b, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: c, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: c, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: d, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: a, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                //Back
                ModelVertex { position: e, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: h, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: f, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: f, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: h, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: g, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                //Top
                ModelVertex { position: c, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: g, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: d, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: d, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: g, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: h, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                //Bottom
                ModelVertex { position: b, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: f, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: a, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: a, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: f, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: e, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                //Left
                ModelVertex { position: a, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: d, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: e, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: e, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: d, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: h, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                //Right
                ModelVertex { position: f, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: g, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: b, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: b, tex_coords: [0.0 , 0.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: g, tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                ModelVertex { position: c, tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
            ]);
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices[..]),
            usage: wgpu::BufferUsage::VERTEX
        });

        Option::Some(Self {
            name: model.id.clone(),
            textures,
            model: if is_cube {
                BlockModel::Cube(vertex_buffer, vertices.len() as u16)
            } else {
                BlockModel::Custom(vertex_buffer, vertices.len() as u16)
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

#[derive(Clone, Copy)]
pub struct BlockState<'block> {
    pub block: &'block dyn Block,
    pub direction: BlockDirection,
    pub damage: u8
}