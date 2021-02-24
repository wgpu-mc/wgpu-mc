use crate::mc::entity::Entity;
use crate::mc::chunk::ChunkManager;
use std::collections::HashMap;
use crate::model::Material;
use crate::mc::block::{Block, StaticBlock};
use std::path::{PathBuf, Path};
use image::{DynamicImage, GenericImageView, Rgba, ColorType};
use crate::mc::datapack::{BlockModelData, NamespacedId};
use guillotiere::{SimpleAtlasAllocator, Size, dump_svg, AtlasAllocator, size2};
use guillotiere::euclid::Size2D;
use crate::mc::resource::{ResourceProvider, ResourceType};
use crate::texture::{Texture, UV};
use std::fs;
use std::fs::File;
use image::imageops::overlay;
use image::codecs::png::PngEncoder;
use wgpu::{TextureDescriptor, Extent3d, TextureFormat, TextureDimension, BindGroupLayout};
use cgmath::Vector2;

pub mod block;
pub mod chunk;
pub mod entity;
pub mod datapack;
pub mod resource;
pub mod gui;

const ATLAS_DIMENSIONS: i32 = 1024;

pub type TextureManager = HashMap<NamespacedId, UV>;

pub struct Minecraft {
    pub sun_position: f32,
    pub block_indices: HashMap<String, usize>,
    pub blocks: Vec<Box<dyn Block>>,
    pub block_model_data: HashMap<String, BlockModelData>,
    pub chunks: ChunkManager,
    pub entities: Vec<Entity>,
    pub atlas_allocator: AtlasAllocator,
    pub atlas_image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub atlas_material: Option<Material>,

    pub texture_manager: TextureManager
}

impl Minecraft {
    pub fn new() -> Self {
        Minecraft {
            sun_position: 0.0,
            block_indices: HashMap::new(),
            chunks: ChunkManager::new(),
            entities: Vec::new(),
            block_model_data: HashMap::new(),
            atlas_allocator: AtlasAllocator::new(Size2D::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            atlas_image: image::ImageBuffer::new(ATLAS_DIMENSIONS as u32, ATLAS_DIMENSIONS as u32),
            atlas_material: None,
            texture_manager: HashMap::new(),
            blocks: Vec::new()
        }
    }

    pub fn load_block_models(&mut self, root: PathBuf) {
        let models_dir = root.join("models").join("block");
        let models_list = std::fs::read_dir(models_dir.clone()).unwrap();

        let mut model_map = &mut self.block_model_data;

        models_list.for_each(|e| {
            let entry = e.unwrap();

            let path = entry.path();
            let split: Vec<&str> = path.file_name().unwrap().to_str().unwrap().split(".").collect();
            let name = *split.first().unwrap();

            datapack::BlockModelData::deserialize(name, (&models_dir).clone(), model_map);
        });

        let model = model_map.get("minecraft:block/cobblestone").unwrap();

        println!("{:?}", model.textures);
    }
    
    pub fn generate_block_texture_atlas(&mut self, rsp: &dyn ResourceProvider, device: &wgpu::Device, queue: &wgpu::Queue, t_bgl: &BindGroupLayout) {
        let mut textures = HashMap::new();

        self.block_model_data.iter().for_each(|(_, bmd)| {
            bmd.textures.iter().for_each(|(_, ns)| {
                match ns {
                    NamespacedId::Resource(_) => {
                        textures.insert(ns.clone(), ());
                    }
                    _ => {}
                }
            });
        });

        textures.iter().for_each(|(ns, _)| {

            let bytes = rsp.get_bytes(
                ResourceType::Texture,
                ns
            );

            let image = image::load_from_memory(&bytes[..]).unwrap();

            let allocation = self.atlas_allocator.allocate(Size2D::new(image.width() as i32, image.height() as i32)).unwrap();

            overlay(&mut self.atlas_image, &image, allocation.rectangle.min.x as u32, allocation.rectangle.min.y as u32);

            self.texture_manager.insert(ns.clone(), (
                Vector2::new(allocation.rectangle.min.x as f32, allocation.rectangle.min.y as f32),
                Vector2::new(allocation.rectangle.max.x as f32, allocation.rectangle.max.y as f32),
            ));
        });

        let texture = Texture::from_image_raw(device, queue, self.atlas_image.as_ref(), Extent3d {
            width: ATLAS_DIMENSIONS as u32,
            height: ATLAS_DIMENSIONS as u32,
            depth: 1
        }, Some("Texture Atlas")).unwrap();

        self.atlas_material = Some(
            Material::from_texture(device, queue, texture, t_bgl, "Texture Atlas".into())
        );
    }

    pub fn generate_blocks(&mut self, device: &wgpu::Device, rp: &dyn ResourceProvider) {
        let mut block_indices: HashMap<String, usize> = HashMap::new();
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        self.block_model_data.iter().for_each(|(name, block_data)| {
            let block = StaticBlock::from_datapack(device, block_data, rp, &self.texture_manager);

            match block {
                None => {} //If it fails, it's most definitely a template and not an actual block
                Some(some) => {
                    // blocks.insert(name.clone(), Box::new(some));
                    blocks.push(Box::new(some));
                    block_indices.insert(name.clone(), blocks.len()-1);
                }
            }
        });

        self.block_indices = block_indices;
        self.blocks = blocks;
    }
}