use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use arc_swap::ArcSwap;
use cgmath::{Point3, Vector2, Vector3};
use guillotiere::AtlasAllocator;
use guillotiere::euclid::Size2D;
use image::{GenericImageView, ImageFormat, Rgba};
use image::imageops::overlay;
use parking_lot::RwLock;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BufferDescriptor, Extent3d};

use crate::camera::{Camera, UniformMatrixHelper};
use crate::mc::block::{Block, PackedBlockstateKey, BlockstateVariantKey};
use crate::mc::chunk::ChunkManager;
use crate::mc::datapack::{BlockModel, TextureVariableOrResource, NamespacedResource, DatapackContextResolver};
use crate::mc::entity::Entity;
use crate::mc::resource::ResourceProvider;
use crate::model::Material;
use crate::render::atlas::{Atlas, ATLAS_DIMENSIONS, Atlases, TextureManager};
use crate::render::pipeline::RenderPipelinesManager;
use crate::texture::{UV, WgpuTexture};

use self::block::model::BlockstateVariantMesh;
use indexmap::map::IndexMap;
use multi_map::MultiMap;
use crate::mc::block::blockstate::BlockstateVariantDefinitionModel;
use crate::WmRenderer;

pub mod block;
pub mod chunk;
pub mod datapack;
pub mod entity;
pub mod gui;
pub mod resource;

#[derive(Debug)]
pub struct BlockEntry {
    pub index: Option<usize>,
    pub model: BlockModel
}

pub struct BlockManager {
    pub blocks: IndexMap<NamespacedResource, Block>,
    pub models: IndexMap<NamespacedResource, BlockModel>,
    pub baked_block_variants: MultiMap<NamespacedResource, PackedBlockstateKey, BlockstateVariantMesh>
}

impl BlockManager {

    pub fn get_packed_blockstate_key(&self, block_id: &NamespacedResource, variant: &str) -> Option<PackedBlockstateKey> {
        let block: &Block = self.blocks.get(block_id)?;

        Some(((self.blocks.get_index_of(block_id)? as u32 & 0x3FFFFF) << 10) |
        (block.states.get_index_of(variant)? as u32 & 0x3FF))
    }

}

fn get_model_or_deserialize<'a>(models: &'a mut IndexMap<NamespacedResource, BlockModel>, model_id: &NamespacedResource, resource_provider: &dyn ResourceProvider, resolver: &dyn DatapackContextResolver) -> Option<&'a BlockModel> {
    let resolved = resolver.resolve(
        "models",
        model_id
    );

    if models.contains_key(&resolved) {
        return models.get(&resolved);
    }

    let mut model_map = HashMap::new();

    BlockModel::deserialize(
        &resolved,
        resource_provider,
        resolver,
        &mut model_map
    )?;

    model_map.into_iter().for_each(|model| {
        if !models.contains_key(&model.0) {
            models.insert(model.0, model.1);
        }
    });

    models.get(model_id)
}

pub struct MinecraftState {
    pub sun_position: ArcSwap<f32>,

    pub block_manager: RwLock<BlockManager>,

    pub chunks: ChunkManager,
    pub entities: RwLock<Vec<Entity>>,

    pub resource_provider: Arc<dyn ResourceProvider>,
    pub context_resolver: Arc<dyn DatapackContextResolver>,

    pub camera: ArcSwap<Camera>,

    pub uniform_buffer: ArcSwap<wgpu::Buffer>,
    pub uniform_bind_group: ArcSwap<wgpu::BindGroup>,

    pub texture_manager: TextureManager
}

impl MinecraftState {
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        pipelines: &RenderPipelinesManager,
        resource_provider: Arc<dyn ResourceProvider>,
        context_resolver: Arc<dyn DatapackContextResolver>) -> Self {
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<UniformMatrixHelper>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipelines.layouts.matrix_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding())
                }
            ]
        });

        MinecraftState {
            sun_position: ArcSwap::new(Arc::new(0.0)),
            chunks: ChunkManager::new(),
            entities: RwLock::new(Vec::new()),

            texture_manager: TextureManager::new(),

            block_manager: RwLock::new(BlockManager {
                blocks: IndexMap::new(),
                models: IndexMap::new(),
                baked_block_variants: MultiMap::new()
            }),

            camera: ArcSwap::new(Arc::new(Camera::new(1.0))),

            uniform_buffer: ArcSwap::new(Arc::new(uniform_buffer)),
            uniform_bind_group: ArcSwap::new(Arc::new(uniform_bind_group)),

            resource_provider,
            context_resolver,
        }
    }

    pub fn bake_blocks(&self, wm: &WmRenderer) {
        let mut block_manager = self.block_manager.write();

        self.bake_block_models(&mut *block_manager);
        self.generate_block_texture_atlas(wm, &block_manager);
        self.bake_blockstate_meshes(&mut *block_manager);
    }

    ///Loops through all the blocks in the `BlockManager`, and bakes their `BlockModel`s and `BlockstateVariantMesh`es
    fn bake_block_models(&self, block_manager: &mut BlockManager) {
        let mut model_map = HashMap::new();

        block_manager.blocks.iter().for_each(|(name, block): (_, &Block)| {
            block.states.iter().for_each(|(variant_key, variant_definition): (&BlockstateVariantKey, &BlockstateVariantDefinitionModel)| {
                let model_resource = &variant_definition.model;

                BlockModel::deserialize(
                    &self.context_resolver.resolve(
                        "models",
                        model_resource
                    ),
                    &*self.resource_provider,
                    &*self.context_resolver,
                    &mut model_map
                );;
            });
        });

        block_manager.models = model_map.into_iter().collect();
    }

    fn generate_block_texture_atlas(
        &self,
        renderer: &WmRenderer,
        block_manager: &BlockManager
    ) -> Option<()> {
        let mut textures = HashSet::new();
        block_manager.models.iter().for_each(|(id, model)| {
            model.textures.iter().for_each(|(key, texture)| {
                match texture {
                    TextureVariableOrResource::Tag(_) => {}
                    TextureVariableOrResource::Resource(res) => {
                        textures.insert(res);
                    }
                }
            });
        });

        let mut atlases = Atlases {
            block: Atlas::new(),
            gui: Atlas::new()
        };

        for id in &textures {
            let bytes = self.resource_provider.get_resource(
                &self.context_resolver.resolve("textures", &id)
            );
            atlases.block.allocate(id, &bytes[..]).unwrap();
        }

        let texture = WgpuTexture::from_image_raw(
            &renderer.wgpu_state.device,
            &renderer.wgpu_state.queue,
            atlases.block.image.as_ref(),
            Extent3d {
                width: ATLAS_DIMENSIONS as u32,
                height: ATLAS_DIMENSIONS as u32,
                depth_or_array_layers: 1,
            },
            Some("Block Texture Atlas"),
        ).ok()?;

        atlases.block.material = Some(Material::from_texture(
            &renderer.wgpu_state.device,
            &renderer.wgpu_state.queue,
            texture,
            &renderer.pipelines.load().layouts.texture_bind_group_layout,
            "Block Texture Atlas".into(),
        ));

        self.texture_manager.atlases.store(Arc::new(atlases));

        Some(())
    }

    fn bake_blockstate_meshes(&self, block_manager: &mut BlockManager) {
        let mut variants = MultiMap::new();

        // let mut models: HashSet<&NamespacedResource> = HashSet::new();
        //
        // block_manager.blocks.iter().for_each(|(name, block)| {
        //     block.states.iter().for_each(|(_, state)| {
        //         models.insert(&state.model);
        //     });
        // });
        //
        // let models: HashMap<&NamespacedResource, &BlockModel> = models.iter().map(|&model| {
        //     (
        //         model.clone(),
        //         block_manager.get_model_or_deserialize(model, &*self.resource_provider).unwrap()
        //     )
        // }).collect();

        block_manager.blocks.iter().for_each(|(name, block)| {
            block.states.iter().for_each(|(key, state)| {
                let block_model = get_model_or_deserialize(
                    &mut block_manager.models,
                    &state.model,
                    &*self.resource_provider,
                    &*self.context_resolver
                ).expect(&format!("{:?}", state.model));

                let mesh = BlockstateVariantMesh::bake_block_model(
                    block_model,
                    &*self.resource_provider,
                    &self.texture_manager,
                    &state.rotations
                ).expect(&format!("{}", name));

                let variant_resource = name.append(&format!("#{}", &key));

                let u32_variant_key: u32 = (
                    (block_manager.blocks.get_index_of(name).unwrap() as u32 & 0x3FFFFF) << 10) |
                    (block.states.get_index_of(key).unwrap() as u32 & 0x3FF);

                variants.insert(variant_resource, u32_variant_key, mesh);
            });
        });

        // println!("{:?}", variants);

        block_manager.baked_block_variants = variants;
    }
}