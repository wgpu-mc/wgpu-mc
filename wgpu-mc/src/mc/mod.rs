use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::mc::block::{Block, StaticBlock};
use crate::mc::chunk::ChunkManager;
use crate::mc::datapack::{BlockModel, Identifier};
use crate::mc::entity::Entity;
use crate::model::Material;
use crate::texture::{WgpuTexture, UV};

use cgmath::{Vector2, Point3, Vector3};
use guillotiere::euclid::Size2D;
use guillotiere::AtlasAllocator;
use image::imageops::overlay;
use image::{GenericImageView, Rgba, ImageFormat};
use wgpu::{BindGroupLayout, Extent3d, BufferDescriptor, BindGroupDescriptor, BindGroupEntry};
use crate::camera::{Camera, Uniforms};
use std::mem::size_of;
use crate::render::pipeline::RenderPipelinesManager;
use crate::render::atlas::{TextureManager, ATLAS_DIMENSIONS};
use std::rc::Rc;
use std::sync::Arc;
use crate::mc::resource::ResourceProvider;
use parking_lot::RwLock;
use crate::ShaderProvider;

pub mod block;
pub mod chunk;
pub mod datapack;
pub mod entity;
pub mod gui;
pub mod resource;

pub struct BlockEntry {
    pub index: Option<usize>,
    pub model: BlockModel
}

pub struct BlockManager {
    ///Blocks should be discovered then put into this map. Once they've all been loaded,
    /// they should be baked into their respective Block trait implementation, and inserted into the
    /// block_array field
    pub registered_blocks: HashSet<Identifier>,
    pub blocks: HashMap<Identifier, BlockEntry>,

    ///For faster indexing
    pub block_array: Vec<Box<dyn Block>>,
}

impl BlockManager {

    pub fn register_block(&mut self, block: Identifier) {
        self.registered_blocks.insert(block);
    }

}

pub struct MinecraftState {
    pub sun_position: f32,

    ///Usually I would simply use a DashMap, but considering that the states of the two fields are intertwined,
    /// this has to be behind a single RwLock
    pub block_manager: RwLock<BlockManager>,

    pub chunks: RwLock<ChunkManager>,
    pub entities: RwLock<Vec<Entity>>,

    pub resource_provider: Arc<dyn ResourceProvider>,
    pub shader_provider: Arc<dyn ShaderProvider>,
    
    pub camera: RwLock<Camera>,

    pub uniform_buffer: RwLock<wgpu::Buffer>,
    pub uniform_bind_group: RwLock<wgpu::BindGroup>,

    pub texture_manager: TextureManager
}

impl MinecraftState {
    #[must_use]
    pub fn new(device: &wgpu::Device, pipelines: &RenderPipelinesManager, resource_provider: Arc<dyn ResourceProvider>, shader_provider: Arc<dyn ShaderProvider>) -> Self {
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

        MinecraftState {
            sun_position: 0.0,
            chunks: RwLock::new(ChunkManager::new()),
            entities: RwLock::new(Vec::new()),

            texture_manager: TextureManager::new(),

            block_manager: RwLock::new(BlockManager {
                registered_blocks: HashSet::new(),
                blocks: HashMap::new(),
                block_array: Vec::new()
            }),

            camera: RwLock::new(Camera::new(1.0)),

            uniform_buffer: RwLock::new(uniform_buffer),
            uniform_bind_group: RwLock::new(uniform_bind_group),

            resource_provider,
            shader_provider
        }
    }

    ///Loops through all the blocks in the `BlockManager`, and creates their respective `BlockModel`
    pub fn generate_block_models(&self) {
        let mut block_manager = self.block_manager.write();
        let mut model_map = HashMap::new();

        block_manager.registered_blocks.iter().for_each(|identifier| {
            datapack::BlockModel::deserialize(identifier, self.resource_provider.as_ref(), &mut model_map);
        });

        model_map.into_iter().for_each(|(identifier, model)| {
            let index = match block_manager.blocks.get(&identifier) {
                None => None,
                Some(entry) => entry.index
            };

            block_manager.blocks.insert(identifier, BlockEntry {
                index,
                model
            });
        });
    }

    pub fn generate_block_texture_atlas(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        t_bgl: &BindGroupLayout,
    ) -> Option<()> {
        let mut textures = HashSet::new();
        let block_manager = self.block_manager.read();

        for (id, entry) in &block_manager.blocks {
            for texture_id in entry.model.textures.values() {
                if let Identifier::Resource(_) = texture_id {
                    textures.insert(texture_id);
                }
            }
        }

        let mut atlases = self.texture_manager.atlases.write();

        for &id in &textures {
            let bytes = self.texture_manager.textures.get(id)?;

            atlases.block.allocate(id, &bytes[..])?;
        }

        let texture = WgpuTexture::from_image_raw(
            device,
            queue,
            atlases.block.image.as_ref(),
            Extent3d {
                width: ATLAS_DIMENSIONS as u32,
                height: ATLAS_DIMENSIONS as u32,
                depth_or_array_layers: 1,
            },
            Some("Block Texture Atlas"),
        ).ok()?;

        atlases.block.material = Some(Material::from_texture(
            device,
            queue,
            texture,
            t_bgl,
            "Block Texture Atlas".into(),
        ));

        Some(())
    }

    pub fn bake_blocks(&self, device: &wgpu::Device) {
        let mut blocks = HashMap::new();
        let mut block_array: Vec<Box<dyn Block>> = Vec::new();

        let mut block_manager = self.block_manager.write();

        for block_data in block_manager.blocks.values_mut() {
            if let Some(block) =
                StaticBlock::from_datapack(device, &block_data.model, self.resource_provider.as_ref(), &self.texture_manager)
            {
                block_array.push(Box::new(block));
                block_data.index = Some(block_array.len() - 1);
            }
        }

        block_manager.blocks = blocks;
        block_manager.block_array = block_array;
    }
}