use std::collections::{HashMap, HashSet};
use std::mem::size_of;


use std::sync::Arc;

use arc_swap::ArcSwap;





use parking_lot::RwLock;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferDescriptor, Extent3d};

use crate::camera::{Camera, UniformMatrixHelper};
use crate::mc::block::{Block, PackedBlockstateKey, BlockstateVariantKey};
use crate::mc::chunk::ChunkManager;
use crate::mc::datapack::{BlockModel, TextureVariableOrResource, NamespacedResource};
use crate::mc::entity::EntityModel;
use crate::mc::resource::ResourceProvider;
use crate::model::BindableTexture;
use crate::render::atlas::{Atlas, ATLAS_DIMENSIONS, TextureManager};
use crate::render::pipeline::RenderPipelineManager;
use crate::texture::{TextureSamplerView};

use self::block::model::BlockstateVariantMesh;
use indexmap::map::IndexMap;
use multi_map::MultiMap;
use crate::mc::block::blockstate::BlockstateVariantDefinitionModel;
use crate::{WgpuState, WmRenderer};

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

fn get_model_or_deserialize<'a>(models: &'a mut IndexMap<NamespacedResource, BlockModel>, model_id: &NamespacedResource, resource_provider: &dyn ResourceProvider) -> Option<&'a BlockModel> {
    if models.contains_key(model_id) {
        return models.get(model_id);
    }

    let mut model_map = HashMap::new();

    BlockModel::deserialize(
        model_id,
        resource_provider,
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
    pub entities: RwLock<Vec<EntityModel>>,

    pub resource_provider: Arc<dyn ResourceProvider>,

    pub camera: ArcSwap<Camera>,

    pub camera_buffer: ArcSwap<wgpu::Buffer>,
    pub camera_bind_group: ArcSwap<wgpu::BindGroup>,

    pub texture_manager: TextureManager
}

impl MinecraftState {
    #[must_use]
    pub fn new(
        wgpu_state: &WgpuState,
        pipelines: &RenderPipelineManager,
        resource_provider: Arc<dyn ResourceProvider>) -> Self {

        let camera_buffer = wgpu_state.device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<UniformMatrixHelper>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let camera_bind_group = wgpu_state.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipelines.bind_group_layouts.matrix4,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(camera_buffer.as_entire_buffer_binding())
                }
            ]
        });

        MinecraftState {
            sun_position: ArcSwap::new(Arc::new(0.0)),
            chunks: ChunkManager::new(),
            entities: RwLock::new(Vec::new()),

            texture_manager: TextureManager::new(wgpu_state, pipelines),

            block_manager: RwLock::new(BlockManager {
                blocks: IndexMap::new(),
                models: IndexMap::new(),
                baked_block_variants: MultiMap::new()
            }),

            camera: ArcSwap::new(Arc::new(Camera::new(1.0))),

            camera_buffer: ArcSwap::new(Arc::new(camera_buffer)),
            camera_bind_group: ArcSwap::new(Arc::new(camera_bind_group)),

            resource_provider
        }
    }

    pub fn bake_blocks(&self, wm: &WmRenderer) {
        let mut block_manager = self.block_manager.write();

        self.generate_block_models(&mut *block_manager);
        self.generate_block_texture_atlas(wm, &block_manager);
        self.bake_blockstate_meshes(&mut *block_manager);
    }

    ///Loops through all the blocks in the `BlockManager`, and generates their `BlockModel`s
    fn generate_block_models(&self, block_manager: &mut BlockManager) {
        let mut model_map = HashMap::new();

        block_manager.blocks.iter().for_each(|(_name, block): (_, &Block)| {
            block.states.iter().for_each(|(_variant_key, variant_definition): (&BlockstateVariantKey, &BlockstateVariantDefinitionModel)| {
                let model_resource = &variant_definition.model;

                BlockModel::deserialize(
                    model_resource,
                    &*self.resource_provider,
                    &mut model_map
                );
            });
        });

        block_manager.models = model_map.into_iter().collect();
    }

    fn generate_block_texture_atlas(
        &self,
        wm: &WmRenderer,
        block_manager: &BlockManager
    ) {
        let mut textures = HashSet::new();
        block_manager.models.iter().for_each(|(_id, model)| {
            model.textures.iter().for_each(|(_key, texture)| {
                match texture {
                    TextureVariableOrResource::Tag(_) => {}
                    TextureVariableOrResource::Resource(res) => {
                        textures.insert(res);
                    }
                }
            });
        });

        let block_atlas = Atlas::new(&*wm.wgpu_state, &*wm.pipelines.load_full());
        //TODO: this goes somewhere else, and do we even need it?
        let gui_atlas = Atlas::new(&*wm.wgpu_state, &*wm.pipelines.load_full());

        block_atlas.allocate(
            &textures.iter().map(|resource| {
                    (
                        *resource,
                        self.resource_provider.get_resource(
                            &resource.prepend("textures/").append(".png")
                        )
                    )
                }).collect::<Vec<(&NamespacedResource, Vec<u8>)>>()[..]
        );

        block_atlas.upload(wm);

        wm.mc.texture_manager.block_texture_atlas.store(Arc::new(block_atlas));
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
                    &*self.resource_provider
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