use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use std::sync::Arc;

use arc_swap::ArcSwap;

use parking_lot::RwLock;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferDescriptor};

use crate::camera::{Camera, UniformMatrixHelper};
use crate::mc::block::{BlockDefinition, BlockstateKey, BlockDefinitionType, Multipart};
use crate::mc::chunk::ChunkManager;
use crate::mc::resource::ResourceProvider;

use crate::render::atlas::{Atlas, TextureManager};
use crate::render::pipeline::RenderPipelineManager;

use self::block::model::BlockModelMesh;
use crate::mc::block::blockstate::{BlockModelRotations, BlockstateVariantModelDefinition};
use crate::render::pipeline::terrain::BLOCK_ATLAS_NAME;
use crate::{WgpuState, WmRenderer};
use indexmap::map::IndexMap;
use crate::mc::block::model::{BlockVariant, BlockStateDefinitionType};
use crate::mc::block::multipart_json::MultipartJson;
use crate::mc::datapack::{AnimationData, BlockModel, NamespacedResource, TextureVariableOrResource};
use crate::mc::entity::Entity;

pub mod block;
pub mod chunk;
pub mod datapack;
pub mod entity;
pub mod gui;
pub mod resource;

#[derive(Debug)]
pub struct BlockEntry {
    pub index: Option<usize>,
    pub model: BlockModel,
}

///Manages everything related to block states.
///
/// Example usage
///
///```
/// use std::convert::TryInto;
/// use wgpu_mc::mc::block::BlockDefinition;
/// use wgpu_mc::mc::datapack::NamespacedResource;
/// use wgpu_mc::WmRenderer;use wgpu_mc::WmRenderer;
/// let wm: WmRenderer;
///
/// let block_manager = wm.mc.block_manager.write();
/// let blockstate_json = r#"
/// {
///    "variants": { ... },
///    "textures": { ...}
/// }
/// "#;
///
/// block_manager.block_definitions.insert(
///    "wgpu_mc_example:blockstates/example_block.json".try_into().unwrap(),
///    BlockDefinition::from_json(blockstate_json).unwrap()
/// );
///
/// //The formatter here is used to define how you would like the blockstates to be named when wgpu-mc
/// //populates the [BlockManager] field
/// fn formatter(resource: &NamespacedResource, state_key: &str) -> String {
///    format!("{}#{}", resource, state_key)
/// }
///
/// wm.mc.bake_blocks(&wm, formatter);
/// ```
pub struct BlockManager {
    ///This is the first field that should be populated when providing wgpu-mc with block info.
    pub block_definitions: HashMap<NamespacedResource, BlockDefinition>,
    ///A map of block models (not meshes) which can be baked into meshes
    pub models: HashMap<NamespacedResource, BlockModel>,
    ///Same as the models field but instead baked into meshes which are ready to be used in rendering
    pub model_meshes: HashMap<NamespacedResource, Arc<BlockModelMesh>>,
    ///[BlockstateKey]s are indices into this Vec. The first element is the block state keys and values, for example
    /// `Block{minecraft:anvil}[facing=west]` would have the field populated as
    /// `{ "facing": "west" }`
    /// and the second element is an [Arc<Block>] which is then
    /// accessed when figuring out how the block should end up being rendered. If the block is varianted (thus not multipart)
    /// then the HashMap will be used directly as a key to get the mesh. If the Block is multipart, it will be matched
    /// against the [MultipartCase]s, and the mesh will be created dynamically as such.
    pub block_states: Vec<(HashMap<String, String>, Arc<BlockVariant>)>,
    /// A HashMap that provides indices into the block_keys field
    pub block_state_indices: HashMap<String, usize>,
}

fn get_model_or_deserialize<'a>(
    models: &'a mut HashMap<NamespacedResource, BlockModel>,
    model_id: &NamespacedResource,
    resource_provider: &dyn ResourceProvider,
) -> Option<&'a BlockModel> {
    if models.contains_key(model_id) {
        return models.get(model_id);
    }

    let mut model_map = HashMap::new();

    BlockModel::deserialize(model_id, resource_provider, &mut model_map)?;

    model_map.into_iter().for_each(|model| {
        if !models.contains_key(&model.0) {
            models.insert(model.0, model.1);
        }
    });

    models.get(model_id)
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
                block_definitions: HashMap::new(),
                models: HashMap::new(),
                model_meshes: HashMap::new(),
                block_states: Vec::new(),
                block_state_indices: HashMap::new(),
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

    ///Create and register all block models from the [Block]s that have been registered
    pub fn bake_blocks<T: Fn(&NamespacedResource, &str) -> String>(&self, wm: &WmRenderer, block_state_name_formatter: T) {
        let mut block_manager = self.block_manager.write();

        self.generate_block_models(&mut *block_manager);
        self.generate_block_texture_atlas(wm, &block_manager);
        self.bake_blockstates(&mut *block_manager, block_state_name_formatter);
    }

    fn generate_block_models(&self, block_manager: &mut BlockManager) {
        let mut models = HashSet::new();

        block_manager
            .block_definitions
            .iter()
            .for_each(|(_name, block): (_, &BlockDefinition)| {
                match &block.definition {
                    BlockDefinitionType::Multipart { multipart } => {
                        multipart.cases.iter().for_each(|case| {
                            case.apply.iter().for_each(|apply| {
                                models.insert(NamespacedResource::try_from(&apply.model).unwrap());
                            });
                        });
                    }
                    BlockDefinitionType::Variants { states } => {
                        states.iter().for_each(
                            |(_variant_key, variant_definition): (
                                &String,
                                &BlockstateVariantModelDefinition,
                            )| {
                                models.insert(variant_definition.model.clone());
                            },
                        );
                    }
                }
            });

        let mut model_map = HashMap::new();

        models.iter().for_each(|model_name| {
            BlockModel::deserialize(
                model_name,
                &*self.resource_provider,
                &mut model_map,
            );
        });

        block_manager.models = model_map;
    }

    fn generate_block_texture_atlas(&self, wm: &WmRenderer, block_manager: &BlockManager) {
        let mut textures = HashSet::new();
        block_manager.models.iter().for_each(|(_id, model)| {
            model
                .textures
                .iter()
                .for_each(|(_key, texture)| match texture {
                    TextureVariableOrResource::Tag(_) => {}
                    TextureVariableOrResource::Resource(res) => {
                        textures.insert(res);
                    }
                });
        });

        let block_atlas = Atlas::new(&*wm.wgpu_state, &*wm.render_pipeline_manager.load_full());

        block_atlas.allocate(
            &textures
                .iter()
                .map(|resource| {
                    let res = &resource.prepend("textures/").append(".png");
                    (
                        *resource,
                        self.resource_provider
                            .get_resource(&resource.prepend("textures/").append(".png")).unwrap(),
                                   AnimationData::deserialize(res, self.resource_provider.borrow()),
                    )
                })
                .collect::<Vec<(&NamespacedResource, Vec<u8>, Option<AnimationData>)>>()[..],
        );

        block_atlas.upload(wm);

        wm.mc
            .texture_manager
            .atlases
            .load_full()
            .get(BLOCK_ATLAS_NAME)
            .unwrap()
            .store(Arc::new(block_atlas));
    }

    fn bake_blockstates<T: Fn(&NamespacedResource, &str) -> String>(&self, block_manager: &mut BlockManager, block_name_formatter: T) {
        let mut block_states = Vec::new();
        let mut indices = HashMap::new();

        let mut multiparts = Vec::new();

        block_manager.block_definitions.iter().for_each(|(name, block): (&NamespacedResource, &BlockDefinition)| {
            match &block.definition {
                BlockDefinitionType::Multipart { multipart } => {
                    let formatted_name = NamespacedResource::try_from(
                        &block_name_formatter(name, "")
                    ).unwrap();

                    multiparts.push(
                        (formatted_name, multipart)
                    );
                }
                BlockDefinitionType::Variants { states } => {
                    states.iter().for_each(|(key, state)| {
                        let block_model = get_model_or_deserialize(
                            &mut block_manager.models,
                            &state.model,
                            &*self.resource_provider,
                        ).unwrap();

                        let mesh = BlockModelMesh::bake_block_model(
                            block_model,
                            &*self.resource_provider,
                            &self.texture_manager,
                            &state.rotations,
                        ).unwrap();

                        let block_state_name = block_name_formatter(name, key);

                        let block = Arc::new(BlockVariant {
                            name: (&block_state_name).try_into().unwrap(),
                            transparent_or_complex: mesh.transparent_or_complex,
                            kind: BlockStateDefinitionType::Variant(mesh),
                        });

                        indices.insert(block_state_name, block_states.len());
                        block_states.push((HashMap::new(), block.clone()));
                    });
                }
            }
        });

        let multipart_models: HashSet<&String> = multiparts.iter().map(|(name, multipart)| {
            multipart.cases.iter().map(|case| {
                case.apply.iter().map(|apply| {
                    &apply.model
                })
            })
        }).flatten().flatten().collect();

        multipart_models.iter().for_each(|&model| {
            let block_model = get_model_or_deserialize(
                &mut block_manager.models,
                &model.try_into().unwrap(),
                &*self.resource_provider,
            ).unwrap();

            let mesh = BlockModelMesh::bake_block_model(
                block_model,
                &*self.resource_provider,
                &self.texture_manager,
                &BlockModelRotations {
                    x: 0,
                    y: 0,
                    z: 0
                }
            ).unwrap();

            block_manager.model_meshes.insert(model.try_into().unwrap(), Arc::new(mesh));
        });

        multiparts.into_iter().for_each(|(name, multipart)| {
            indices.insert(
                block_name_formatter(&name, ""),
                block_states.len()
            );

            block_states.push(
                (
                    HashMap::new(),
                    Arc::new(BlockVariant {
                        name,
                        kind: BlockStateDefinitionType::Multipart(Arc::new(
                            Multipart::from_json(multipart, &block_manager)
                        )),
                        transparent_or_complex: true
                    })
                )
            );
        });

        block_manager.block_state_indices = indices;
        block_manager.block_states = block_states;
    }
}
