use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use std::sync::Arc;

use arc_swap::ArcSwap;

use minecraft_assets::schemas;
use parking_lot::RwLock;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferDescriptor};

use crate::camera::{Camera, UniformMatrixHelper};
use crate::mc::block::{BlockstateKey};
use crate::mc::chunk::ChunkManager;
use crate::mc::resource::ResourceProvider;

use crate::render::atlas::{Atlas, TextureManager};
use crate::render::pipeline::RenderPipelineManager;

use crate::render::pipeline::terrain::BLOCK_ATLAS_NAME;
use crate::{WgpuState, WmRenderer};
use indexmap::map::IndexMap;
use crate::mc::entity::Entity;

use self::block::ModelMesh;
use self::resource::ResourcePath;

pub mod block;
pub mod chunk;
pub mod entity;
pub mod resource;

///Take in a block name (not a [ResourcePath]!) and optionally a variant state key, e.g. "facing=north" and format it some way
/// for example, `minecraft:anvil[facing=north]` or `Block{minecraft:anvil}[facing=north]`
pub type BlockVariantFormatter = dyn Fn(&str, Option<&str>) -> String;

pub struct BlockManager {
    ///This maps block state keys to either a [VariantMesh] or a [Multipart] struct. How the keys are formatted
    /// is defined by the user of wgpu-mc. For example `Block{minecraft:anvil}[facing=west]` or `minecraft:anvil#facing=west`
    pub blocks: IndexMap<String, Block>
}

#[derive(Debug)]
pub enum Block {
    Multipart(Multipart),
    Variants(IndexMap<String, Arc<ModelMesh>>)
}

impl Block {

    pub fn get_model(&self, key: u16) -> Arc<ModelMesh> {
        match &self {
            Block::Multipart(multipart) => {
                multipart.keys.read().get_index(key as usize).unwrap().1.clone()
            },
            Block::Variants(variants) => {
                variants.get_index(key as usize).unwrap().1.clone()
            }
        }
    }

    pub fn get_model_by_key<'a>(
        &self, 
        key: impl IntoIterator<Item = (&'a str, &'a schemas::blockstates::multipart::StateValue)> + Clone,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas
    ) -> (Arc<ModelMesh>, u16) {
        let key_string = key.clone().into_iter()
                    .map(|(key, value)| format!("{}={}", key, match value {
                        schemas::blockstates::multipart::StateValue::Bool(bool) => if *bool { "true" } else { "false" },
                        schemas::blockstates::multipart::StateValue::String(string) => &string,
                    }))
                    .collect::<Vec<String>>()
                    .join(",");

        match &self {
            Block::Multipart(multipart) => {
                {
                    match multipart.keys.read().get_full(&key_string) {
                        Some(full) => return (full.2.clone(), full.0 as u16),
                        None => {}
                    }
                }

                let mesh = multipart.generate_mesh(key, resource_provider, block_atlas);

                let mut multipart_write = multipart.keys.write();
                multipart_write.insert(
                    key_string, 
                    mesh.clone()
                );
                
                (mesh, multipart_write.len() as u16 - 1)
            },
            Block::Variants(variants) => {
                let full = variants.get_full(&key_string).unwrap();
                (full.2.clone(), full.0 as u16)
            }
        }
    }
}

#[derive(Debug)]
pub struct Multipart {
    pub cases: Vec<schemas::blockstates::multipart::Case>,
    pub keys: RwLock<IndexMap<String, Arc<ModelMesh>>>
}

impl Multipart {

    pub fn generate_mesh<'a>(
        &self, 
        key: impl IntoIterator<Item = (&'a str, &'a schemas::blockstates::multipart::StateValue)> + Clone, 
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas
    ) -> Arc<ModelMesh> {
        let apply_variants = self.cases.iter()
            .filter_map(|case| {
                if case.applies(key.clone()) {
                    Some(&case.apply)
                } else {
                    None
                }
            });
        
        let mesh = ModelMesh::bake(apply_variants, resource_provider, block_atlas)
            .unwrap();
        
        Arc::new(mesh)
    }

}
pub enum MultipartOrMesh {
    Multipart(Arc<Multipart>),
    Mesh(Arc<ModelMesh>)
}

///Multipart models are generated dynamically as they can be too complex 
pub struct BlockInstance {
    pub render_settings: block::RenderSettings,
    pub block: MultipartOrMesh
}

///Minecraft-specific state and data structures go in here
pub struct MinecraftState {
    pub sun_position: ArcSwap<f32>,

    pub block_manager: RwLock<BlockManager>,

    pub chunks: ChunkManager,
    pub entity_models: RwLock<Vec<Entity>>,

    pub resource_provider: Arc<dyn ResourceProvider>,

    pub camera: ArcSwap<Camera>,

    pub camera_buffer: ArcSwap<Option<wgpu::Buffer>>,
    pub camera_bind_group: ArcSwap<Option<wgpu::BindGroup>>,

    pub texture_manager: TextureManager,

    pub animated_block_buffer: ArcSwap<Option<wgpu::Buffer>>,
    pub animated_block_bind_group: ArcSwap<Option<wgpu::BindGroup>>,
}

impl MinecraftState {

    #[must_use]
    pub fn new(
        resource_provider: Arc<dyn ResourceProvider>,
    ) -> Self {
        MinecraftState {
            sun_position: ArcSwap::new(Arc::new(0.0)),
            chunks: ChunkManager::new(),
            entity_models: RwLock::new(Vec::new()),

            texture_manager: TextureManager::new(),

            block_manager: RwLock::new(BlockManager {
                blocks: IndexMap::new()
            }),

            camera: ArcSwap::new(Arc::new(Camera::new(1.0))),

            camera_buffer: ArcSwap::new(Arc::new(None)),
            camera_bind_group: ArcSwap::new(Arc::new(None)),

            resource_provider,

            animated_block_buffer: ArcSwap::new(Arc::new(None)),
            animated_block_bind_group: ArcSwap::new(Arc::new(None)),
        }
    }

    pub fn init_camera(&self, wm: &WmRenderer) {
        let camera_buffer = wm.wgpu_state.device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<UniformMatrixHelper>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = wm
            .wgpu_state
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: wm
                    .render_pipeline_manager
                    .load()
                    .bind_group_layouts
                    .read()
                    .get("matrix4")
                    .unwrap(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        camera_buffer.as_entire_buffer_binding(),
                    ),
                }],
            });

        self.camera_buffer.store(Arc::new(Some(camera_buffer)));
        self.camera_bind_group
            .store(Arc::new(Some(camera_bind_group)))
    }

    ///Bake blocks from their blockstates
    ///
    /// # Example
    ///
    /// ```
    /// use wgpu_mc::mc::MinecraftState;
    ///
    /// let minecraft_state: MinecraftState;
    /// let wm: WmRenderer;
    /// 
    /// minecraft_state.bake_blocks(
    ///     &wm, 
    ///     ["minecraft:anvil", "minecraft:blockstates/anvil.json".into()]
    /// );
    /// ```
    pub fn bake_blocks<'a>(
        &self, 
        wm: &WmRenderer, 
        block_states: impl IntoIterator<Item = (impl AsRef<str>, &'a ResourcePath)>
    ) {
        let mut block_manager = self.block_manager.write();
        let block_atlas = self.texture_manager.atlases.load()
            .get(BLOCK_ATLAS_NAME)
            .unwrap()
            .load();

        //Figure out which block models there are
        block_states.into_iter().for_each(|(block_name, block_state)| {
            let blockstates: schemas::BlockStates = serde_json::from_str(
                &self.resource_provider.get_string(block_state).unwrap()
            ).unwrap();

            let block = match &blockstates {
                schemas::BlockStates::Variants { variants } => {
                    let meshes: IndexMap<String, Arc<ModelMesh>> = variants.iter().map(|(variant_id, variant)| {
                        let mesh = ModelMesh::bake([variant], &*self.resource_provider, &block_atlas)
                            .unwrap();
                        (variant_id.clone(), Arc::new(mesh))
                    }).collect();

                    Block::Variants(meshes)
                },
                schemas::BlockStates::Multipart { cases } => {
                    Block::Multipart(Multipart {
                        cases: cases.clone(),
                        keys: RwLock::new(IndexMap::new()),
                    })
                },
            };

            block_manager.blocks.insert(String::from(block_name.as_ref()), block);
        });

        block_atlas.upload(&wm);

    }

}
