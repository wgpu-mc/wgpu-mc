use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::mc::block::{Block, StaticBlock};
use crate::mc::chunk::ChunkManager;
use crate::mc::datapack::{BlockModelData, NamespacedId};
use crate::mc::entity::Entity;
use crate::mc::resource::{ResourceProvider, ResourceType};
use crate::model::Material;
use crate::texture::{WgTexture, UV};

use cgmath::Vector2;
use guillotiere::euclid::Size2D;
use guillotiere::AtlasAllocator;
use image::imageops::overlay;
use image::{GenericImageView, Rgba};
use wgpu::{BindGroupLayout, Extent3d};

pub mod block;
pub mod chunk;
pub mod datapack;
pub mod entity;
pub mod gui;
pub mod resource;

const ATLAS_DIMENSIONS: i32 = 1024;

pub type TextureManager = HashMap<NamespacedId, UV>;

pub struct MinecraftRenderer {
    pub sun_position: f32,
    pub block_indices: HashMap<String, usize>,
    pub blocks: Vec<Box<dyn Block>>,
    pub block_model_data: HashMap<String, BlockModelData>,
    pub chunks: ChunkManager,
    pub entities: Vec<Entity>,

    pub block_atlas_allocator: AtlasAllocator,
    pub block_atlas_image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub block_atlas_material: Option<Material>,

    pub gui_atlas_allocator: AtlasAllocator,
    pub gui_atlas_image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub gui_atlas_material: Option<Material>,

    pub texture_manager: TextureManager,
}

impl MinecraftRenderer {
    pub fn new() -> Self {
        MinecraftRenderer {
            sun_position: 0.0,
            block_indices: HashMap::new(),
            chunks: ChunkManager::new(),
            entities: Vec::new(),
            block_model_data: HashMap::new(),
            block_atlas_allocator: AtlasAllocator::new(Size2D::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            block_atlas_image: image::ImageBuffer::new(
                ATLAS_DIMENSIONS as u32,
                ATLAS_DIMENSIONS as u32,
            ),

            block_atlas_material: None,
            gui_atlas_allocator: AtlasAllocator::new(Size2D::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            gui_atlas_image: Default::default(),
            gui_atlas_material: None,
            texture_manager: HashMap::new(),
            blocks: Vec::new(),
        }
    }

    //TODO: make this not suck and also genericize it to not require fs
    pub fn load_block_models(&mut self, root: PathBuf) {
        let models_dir = root.join("models").join("block");
        let models_list = std::fs::read_dir(models_dir.clone()).unwrap();

        let mut model_map = &mut self.block_model_data;

        for e in models_list {
            let entry = e.unwrap();

            let path = entry.path();
            let split: Vec<&str> = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split('.')
                .collect();
            let name = *split.first().unwrap();

            datapack::BlockModelData::deserialize(name, (&models_dir).clone(), &mut model_map);
        }
    }

    pub fn generate_block_texture_atlas(
        &mut self,
        rsp: &dyn ResourceProvider,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        t_bgl: &BindGroupLayout,
    ) {
        let mut textures = HashSet::new();

        for (_, bmd) in self.block_model_data.iter() {
            for (_, ns) in bmd.textures.iter() {
                if let NamespacedId::Resource(_) = ns {
                    textures.insert(ns.clone());
                }
            }
        }

        for ns in textures.iter() {
            let bytes = rsp.get_bytes(ResourceType::Texture, &ns);

            let image = image::load_from_memory(&bytes[..]).unwrap();

            let allocation = self
                .block_atlas_allocator
                .allocate(Size2D::new(image.width() as i32, image.height() as i32))
                .unwrap();

            overlay(
                &mut self.block_atlas_image,
                &image,
                allocation.rectangle.min.x as u32,
                allocation.rectangle.min.y as u32,
            );

            self.texture_manager.insert(
                ns.clone(),
                (
                    Vector2::new(
                        allocation.rectangle.min.x as f32,
                        allocation.rectangle.min.y as f32,
                    ),
                    Vector2::new(
                        allocation.rectangle.max.x as f32,
                        allocation.rectangle.max.y as f32,
                    ),
                ),
            );
        }

        let texture = WgTexture::from_image_raw(
            device,
            queue,
            self.block_atlas_image.as_ref(),
            Extent3d {
                width: ATLAS_DIMENSIONS as u32,
                height: ATLAS_DIMENSIONS as u32,
                depth: 1,
            },
            Some("Block Texture Atlas"),
        )
        .unwrap();

        self.block_atlas_material = Some(Material::from_texture(
            device,
            queue,
            texture,
            t_bgl,
            "Block Texture Atlas".into(),
        ));
    }

    pub fn generate_blocks(&mut self, device: &wgpu::Device, rp: &dyn ResourceProvider) {
        let mut block_indices = HashMap::new();
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        for (name, block_data) in self.block_model_data.iter() {
            if let Some(block) =
                StaticBlock::from_datapack(device, block_data, rp, &self.texture_manager)
            {
                blocks.push(Box::new(block));
                block_indices.insert(name.clone(), blocks.len() - 1);
            }
        }

        self.block_indices = block_indices;
        self.blocks = blocks;
    }
}

impl Default for MinecraftRenderer {
    fn default() -> Self {
        Self::new()
    }
}
