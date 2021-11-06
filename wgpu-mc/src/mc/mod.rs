use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::mc::block::{Block, StaticBlock};
use crate::mc::chunk::ChunkManager;
use crate::mc::datapack::{BlockModelData, Identifier};
use crate::mc::entity::Entity;
use crate::mc::resource::{ResourceProvider, ResourceType};
use crate::model::Material;
use crate::texture::{WgTexture, UV};

use cgmath::{Vector2, Point3, Vector3};
use guillotiere::euclid::Size2D;
use guillotiere::AtlasAllocator;
use image::imageops::overlay;
use image::{GenericImageView, Rgba, ImageFormat};
use wgpu::{BindGroupLayout, Extent3d, BufferDescriptor, BindGroupDescriptor, BindGroupEntry};
use crate::camera::{Camera, Uniforms};
use std::mem::size_of;
use crate::render::pipeline::Pipelines;
use crate::render::atlas::{TextureManager, ATLAS_DIMENSIONS};

pub mod block;
pub mod chunk;
pub mod datapack;
pub mod entity;
pub mod gui;
pub mod resource;

pub struct MinecraftRenderer {
    pub sun_position: f32,
    pub block_indices: HashMap<String, usize>,
    pub blocks: Vec<Box<dyn Block>>,
    pub block_model_data: HashMap<String, BlockModelData>,
    pub chunks: ChunkManager,
    pub entities: Vec<Entity>,
    
    pub camera: Camera,

    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,

    pub texture_manager: TextureManager
}

impl MinecraftRenderer {
    pub fn new(device: &wgpu::Device, pipelines: &Pipelines) -> Self {
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipelines.layouts.camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding())
                }
            ]
        });

        MinecraftRenderer {
            sun_position: 0.0,
            block_indices: HashMap::new(),
            chunks: ChunkManager::new(),
            entities: Vec::new(),
            block_model_data: HashMap::new(),

            texture_manager: TextureManager::new(),

            blocks: Vec::new(),
            camera: Camera::new(1.0),

            uniform_buffer,
            uniform_bind_group
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        t_bgl: &BindGroupLayout,
    ) -> Option<()> {
        let mut textures = HashSet::new();

        for (_, bmd) in self.block_model_data.iter() {
            for (_, ns) in bmd.textures.iter() {
                if let Identifier::Resource(_) = ns {
                    textures.insert(ns.clone());
                }
            }
        }

        for id in textures.iter() {
            let bytes = self.texture_manager.textures.get(id)?;

            self.texture_manager.atlases.block.allocate(id, &bytes[..])?;
        }

        let texture = WgTexture::from_image_raw(
            device,
            queue,
            self.texture_manager.atlases.block.image.as_ref(),
            Extent3d {
                width: ATLAS_DIMENSIONS as u32,
                height: ATLAS_DIMENSIONS as u32,
                depth_or_array_layers: 1,
            },
            Some("Block Texture Atlas"),
        ).ok()?;

        self.texture_manager.atlases.block.material = Some(Material::from_texture(
            device,
            queue,
            texture,
            t_bgl,
            "Block Texture Atlas".into(),
        ));

        Some(())
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