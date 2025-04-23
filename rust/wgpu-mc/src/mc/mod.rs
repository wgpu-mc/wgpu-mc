//! Rust implementations of minecraft concepts that are important to us.

use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use chunk::SectionStorage;
use glam::{ivec2, IVec2};
use indexmap::map::IndexMap;
use minecraft_assets::schemas;
use minecraft_assets::schemas::blockstates::multipart::StateValue;
use parking_lot::{Mutex, RwLock};

use crate::mc::entity::{BundledEntityInstances, Entity};
use crate::mc::resource::ResourceProvider;
use crate::render::atlas::{Atlas, TextureManager};
use crate::render::pipeline::BLOCK_ATLAS;
use crate::util::BindableBuffer;
use crate::{Display, WmRenderer};

use self::block::ModelMesh;
use self::resource::ResourcePath;

pub mod block;
pub mod chunk;
pub mod direction;
pub mod entity;
pub mod resource;
/// Take in a block name (not a [ResourcePath]!) and optionally a variant state key, e.g. "facing=north" and format it some way
/// for example, `minecraft:anvil[facing=north]` or `Block{minecraft:anvil}[facing=north]`
pub type BlockVariantFormatter = dyn Fn(&str, Option<&str>) -> String;

pub struct BlockManager {
    /// This maps block state keys to either a [VariantMesh] or a [Multipart] struct. How the keys are formatted
    /// is defined by the user of wgpu-mc. For example `Block{minecraft:anvil}[facing=west]` or `minecraft:anvil#facing=west`
    pub blocks: IndexMap<String, Block>,
}

#[derive(Debug)]
pub enum Block {
    Multipart(Multipart),
    Variants(IndexMap<Vec<(String, StateValue)>, Vec<Arc<ModelMesh>>>),
}

impl Block {
    pub fn get_model(&self, key: u16, _seed: u8) -> Option<Arc<ModelMesh>> {
        Some(match &self {
            Block::Multipart(multipart) => multipart.keys.read().get_index(key as usize)?.1.clone(),
            //TODO, random variant selection through weight and seed
            Block::Variants(variants) => variants.get_index(key as usize)?.1[0].clone(),
        })
    }

    pub fn get_model_by_key<'a>(
        &self,
        key: impl IntoIterator<Item = (&'a str, &'a StateValue)> + Clone,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas,
        //TODO use this
        _seed: u8,
    ) -> Option<(Arc<ModelMesh>, u16)> {
        let key_map: HashMap<&str, &StateValue> = key.clone().into_iter().collect();

        let key_string = key
            .clone()
            .into_iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    key,
                    match value {
                        StateValue::Bool(bool) =>
                            if *bool {
                                "true"
                            } else {
                                "false"
                            },
                        StateValue::String(string) => string,
                    }
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        match &self {
            Block::Multipart(multipart) => {
                {
                    if let Some(full) = multipart.keys.read().get_full(&key_string) {
                        return Some((full.2.clone(), full.0 as u16));
                    }
                }

                let mesh = multipart.generate_mesh(key, resource_provider, block_atlas);

                let mut multipart_write = multipart.keys.write();
                multipart_write.insert(key_string, mesh.clone());

                Some((mesh, multipart_write.len() as u16 - 1))
            }
            Block::Variants(variants) => {
                let full =
                    variants
                        .iter()
                        .enumerate()
                        .find(|(_, (variant_key, _model_mesh))| {
                            variant_key.iter().all(
                                |(variant_property_key, variant_property_value)| {
                                    key_map
                                        .get(&variant_property_key[..])
                                        .map_or(false, |v| v == &variant_property_value)
                                },
                            )
                        })?;

                Some((full.1 .1[0].clone(), full.0 as u16))
            }
        }
    }
}

#[derive(Debug)]
pub struct Multipart {
    pub cases: Vec<schemas::blockstates::multipart::Case>,
    pub keys: RwLock<IndexMap<String, Arc<ModelMesh>>>,
}

impl Multipart {
    pub fn generate_mesh<'a>(
        &self,
        key: impl IntoIterator<Item = (&'a str, &'a schemas::blockstates::multipart::StateValue)>
            + Clone,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas,
    ) -> Arc<ModelMesh> {
        let apply_variants = self.cases.iter().filter_map(|case| {
            if case.applies(key.clone()) {
                Some(case.apply.models())
            } else {
                None
            }
        });

        let mesh = ModelMesh::bake(
            apply_variants.into_iter().flatten(),
            resource_provider,
            block_atlas,
        )
        .unwrap();

        Arc::new(mesh)
    }
}

pub enum MultipartOrMesh {
    Multipart(Arc<Multipart>),
    Mesh(Arc<ModelMesh>),
}

/// Multipart models are generated dynamically as they can be too complex
pub struct BlockInstance {
    pub render_settings: block::RenderSettings,
    pub block: MultipartOrMesh,
}

#[derive(Default, Clone)]
pub struct SkyState {
    pub color: [f32; 3],
    pub angle: f32,
    pub brightness: f32,
    pub star_shimmer: f32,
    pub moon_phase: i32,
}

#[derive(Default, Clone)]
pub struct RenderEffectsData {
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_shape: f32,
    pub fog_color: [f32; 4],
    pub color_modulator: [f32; 4],
    pub dimension_fog_color: [f32; 4],
}

pub struct Scene {
    pub section_storage: RwLock<SectionStorage>,
    pub camera_section_pos: RwLock<IVec2>,
    pub chunk_buffer: Arc<BindableBuffer>,

    pub indirect_buffer: Arc<wgpu::Buffer>,

    pub entity_instances: Mutex<HashMap<String, BundledEntityInstances>>,
    pub sky_state: ArcSwap<SkyState>,
    
    pub render_effects: ArcSwap<RenderEffectsData>,

    pub depth_texture: RwLock<wgpu::Texture>,
}

impl Scene {
    pub fn new(wm: &WmRenderer, framebuffer_size: wgpu::Extent3d) -> Self {
        let indirect_buffer = wm.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4 * 5 * 10000,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });
        let buffer_size = 100000000u64;
        Self {
            section_storage: RwLock::new(SectionStorage::new((buffer_size / 4) as u32)),
            camera_section_pos: RwLock::new(ivec2(0, 0)),
            chunk_buffer: Arc::new(BindableBuffer::new_deferred(
                wm,
                buffer_size,
                wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDEX,
                "ssbo",
            )),
            indirect_buffer: Arc::new(indirect_buffer),

            entity_instances: Default::default(),
            sky_state: Default::default(),
            render_effects: Default::default(),
            depth_texture: wm
                .gpu
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: framebuffer_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .into(),
        }
    }

    pub fn resize_depth_texture(&self, wm: &WmRenderer, width: u32, height: u32) {
        self.depth_texture.read().destroy();
        *self.depth_texture.write() = wm.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }
}

/// Minecraft-specific state and data structures go in here
pub struct MinecraftState {
    pub block_manager: RwLock<BlockManager>,

    pub entity_models: RwLock<HashMap<String, Arc<Entity>>>,

    pub resource_provider: Arc<dyn ResourceProvider>,
    pub texture_manager: TextureManager,

    pub animated_block_buffer: ArcSwap<Option<wgpu::Buffer>>,
    pub animated_block_bind_group: ArcSwap<Option<wgpu::BindGroup>>,
}

impl MinecraftState {
    #[must_use]
    pub fn new(wgpu_state: &Display, resource_provider: Arc<dyn ResourceProvider>) -> Self {
        MinecraftState {
            entity_models: RwLock::new(HashMap::new()),

            texture_manager: TextureManager::new(wgpu_state),

            block_manager: RwLock::new(BlockManager {
                blocks: IndexMap::new(),
            }),
            resource_provider,

            animated_block_buffer: ArcSwap::new(Arc::new(None)),
            animated_block_bind_group: ArcSwap::new(Arc::new(None)),
        }
    }

    /// Bake blocks from their blockstates
    ///
    /// # Example
    ///
    ///```ignore
    /// # use wgpu_mc::mc::MinecraftState;
    /// # use wgpu_mc::mc::resource::ResourcePath;
    /// # use wgpu_mc::WmRenderer;
    ///
    /// # let minecraft_state: MinecraftState;
    /// # let wm: WmRenderer;
    ///
    /// minecraft_state.bake_blocks(
    ///     &wm,
    ///     [("minecraft:anvil", &ResourcePath("minecraft:blockstates/anvil.json".into()))]
    /// );
    /// ```
    pub fn bake_blocks<'a>(
        &self,
        wm: &WmRenderer,
        block_states: impl IntoIterator<Item = (impl AsRef<str>, &'a ResourcePath)>,
    ) {
        let mut block_manager = self.block_manager.write();
        let atlases = self.texture_manager.atlases.read();
        let block_atlas = atlases.get(BLOCK_ATLAS).unwrap();

        //Figure out which block models there are
        block_states
            .into_iter()
            .for_each(|(block_name, block_state)| {
                let blockstates: schemas::BlockStates =
                    serde_json::from_str(&self.resource_provider.get_string(block_state).unwrap())
                        .unwrap();

                let block = match &blockstates {
                    schemas::BlockStates::Variants { variants } => {
                        let meshes: IndexMap<Vec<(String, StateValue)>, Vec<Arc<ModelMesh>>> =
                            variants
                                .iter()
                                .map(|(variant_id, variant)| {
                                    let key_iter = if !variant_id.is_empty() {
                                        variant_id
                                            .split(',')
                                            .filter_map(|kv_pair| {
                                                let mut split = kv_pair.split('=');
                                                if kv_pair.is_empty() {
                                                    return None;
                                                }

                                                Some((
                                                    split.next().unwrap().to_string(),
                                                    match split.next().unwrap() {
                                                        "true" => StateValue::Bool(true),
                                                        "false" => StateValue::Bool(false),
                                                        other => StateValue::String(other.into()),
                                                    },
                                                ))
                                            })
                                            .collect::<Vec<_>>()
                                    } else {
                                        vec![]
                                    };

                                    (
                                        key_iter,
                                        variant
                                            .models()
                                            .iter()
                                            .map(|variation| {
                                                Arc::new(
                                                    ModelMesh::bake(
                                                        std::slice::from_ref(variation),
                                                        &*self.resource_provider,
                                                        block_atlas,
                                                    )
                                                    .unwrap(),
                                                )
                                            })
                                            .collect::<Vec<Arc<ModelMesh>>>(),
                                    )
                                })
                                .collect();

                        Block::Variants(meshes)
                    }
                    schemas::BlockStates::Multipart { cases } => Block::Multipart(Multipart {
                        cases: cases.clone(),
                        keys: RwLock::new(IndexMap::new()),
                    }),
                };

                block_manager
                    .blocks
                    .insert(String::from(block_name.as_ref()), block);
            });

        block_atlas.upload(wm);
    }
}
