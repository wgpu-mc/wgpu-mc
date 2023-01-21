use bytemuck::{Pod, Zeroable};
use cgmath::{Deg, Matrix2, Matrix3, Matrix4, SquareMatrix, Vector2, Vector3, Vector4};
use minecraft_assets::api::ModelResolver;
use minecraft_assets::schemas;
use serde_derive::{Deserialize, Serialize};

use crate::mc::resource::ResourceProvider;
use crate::render::atlas::{Atlas, ATLAS_DIMENSIONS};
use crate::texture::UV;

use super::resource::ResourcePath;

pub type BlockPos = (i32, u16, i32);

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct BlockstateKey {
    ///An index into [BlockManager]
    pub block: u16,
    ///Used to quickly figure out which [ModelMesh] a [Block] should return without having to hash strings
    pub augment: u16,
}

impl BlockstateKey {
    pub fn pack(&self) -> u32 {
        ((self.block as u32) << 16) | (self.augment as u32)
    }
}

impl From<(u16, u16)> for BlockstateKey {
    fn from(tuple: (u16, u16)) -> Self {
        Self {
            block: tuple.0,
            augment: tuple.1,
        }
    }
}

impl From<u32> for BlockstateKey {
    fn from(int: u32) -> Self {
        Self::from(((int >> 16) as u16, (int & 0xffff) as u16))
    }
}

///The state of one block, describing which variant
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChunkBlockState {
    Air,
    State(BlockstateKey),
}

impl ChunkBlockState {
    pub fn is_air(&self) -> bool {
        matches!(self, Self::Air)
    }
}

///Represents a vertex in a block mesh, including an additional UV offset index for animated textures.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BlockMeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 4],
    pub animation_uv_offset: u32,
}

#[derive(Debug)]
pub struct BlockModelFaces {
    pub north: Option<[BlockMeshVertex; 6]>,
    pub east: Option<[BlockMeshVertex; 6]>,
    pub south: Option<[BlockMeshVertex; 6]>,
    pub west: Option<[BlockMeshVertex; 6]>,
    pub up: Option<[BlockMeshVertex; 6]>,
    pub down: Option<[BlockMeshVertex; 6]>,
}

#[derive(Debug)]
///Makes chunk mesh baking a bit faster
pub enum CubeOrComplexMesh {
    ///Known to be a simple cube. Only cubes are eligible for sides to be culled depending on the state if it's neighbours
    Cube(Box<BlockModelFaces>),
    ///Something other than a simple cube, such as an anvil, slab, or enchanting table.
    Complex(Vec<BlockModelFaces>),
}

fn recurse_model_parents(
    model: &schemas::Model,
    resource_provider: &dyn ResourceProvider,
    models: &mut Vec<ResourcePath>,
) {
    match &model.parent {
        Some(parent_path_string) => {
            let parent_path: ResourcePath = ResourcePath::from(parent_path_string)
                .prepend("models/")
                .append(".json");
            recurse_model_parents(
                &serde_json::from_str(
                    &resource_provider
                        .get_string(&parent_path)
                        .expect(&parent_path.0),
                )
                .unwrap(),
                resource_provider,
                models,
            );
            models.push(parent_path);
        }
        None => {}
    }
}

fn resolve_model(
    model: schemas::Model,
    resource_provider: &dyn ResourceProvider,
) -> schemas::Model {
    if model.parent.is_none() {
        return model;
    }

    let mut parent_paths = Vec::new();
    recurse_model_parents(&model, resource_provider, &mut parent_paths);

    let parents: Vec<schemas::Model> = parent_paths
        .iter()
        .map(|parent_path| {
            serde_json::from_str(&resource_provider.get_string(parent_path).unwrap()).unwrap()
        })
        .collect();

    let mut schema = ModelResolver::resolve_model([&model].into_iter().chain(parents.iter()));

    if let Some(textures) = &mut schema.textures {
        let copy = textures.clone();

        textures.iter_mut().for_each(|(_key, texture)| {
            if texture.reference().is_some() {
                texture.0 = texture.resolve(&copy).unwrap().to_string();
            }
        })
    }

    schema
}

fn get_atlas_uv(face: &schemas::models::ElementFace, block_atlas: &Atlas) -> Option<UV> {
    let atlas_map = block_atlas.uv_map.read();

    let atlas_uv = atlas_map.get(&(&face.texture.0).into())?;

    const ATLAS: f32 = ATLAS_DIMENSIONS as f32;

    let middle_x = atlas_uv.0 .0 + (atlas_uv.1 .0 / 2.0);
    let middle_y = atlas_uv.0 .1 + (atlas_uv.1 .1 / 2.0);

    // let mat = Matrix3::from_translation([middle_x, middle_y].into())
    //     * Matrix3::from_angle_x(Deg((90 * face.rotation) as f32))
    //     * Matrix3::from_translation([-middle_x, -middle_y].into());

    let mat = Matrix3::identity();

    let uv1 = mat * Vector3::new((atlas_uv.0 .0) / ATLAS, (atlas_uv.0 .1) / ATLAS, 1.0);
    let uv2 = mat * Vector3::new((atlas_uv.1 .0) / ATLAS, (atlas_uv.1 .1) / ATLAS, 1.0);

    Some(((uv1.x, uv1.y), (uv2.x, uv2.y)))
}
pub struct RenderSettings {
    pub opaque: bool,
}

#[derive(Debug)]
pub enum MeshBakeError {
    UnresolvedTextureReference(String),
    UnresolvedResourcePath(ResourcePath),
    JsonError(serde_json::Error),
}

///A block model which has been baked into a mesh and is ready for rendering
#[derive(Debug)]
pub struct ModelMesh {
    pub models: Vec<(CubeOrComplexMesh, bool)>,
}

impl ModelMesh {
    pub fn bake<'a>(
        variants: impl IntoIterator<Item = &'a schemas::blockstates::Variant>,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas,
    ) -> Result<Self, MeshBakeError> {
        let models = variants.into_iter()
            .flat_map(|variant| variant.models())
            .map(|model_properties| {
                let model_resource_path = ResourcePath::from(&model_properties.model).prepend("models/").append(".json");

                //Recursively resolve the model using it's parents if it has any
                let model: schemas::Model = resolve_model(
                    //Parse the JSON into the model schema
                    serde_json::from_str(
                        //Get the model JSON
                        &resource_provider.get_string(&model_resource_path)
                            .ok_or_else(|| MeshBakeError::UnresolvedResourcePath(model_resource_path))?
                    ).map_err(MeshBakeError::JsonError)?,
                    resource_provider
                );

                if let Some(textures) = model.textures {
                    //Make sure the textures in the model are fully resolved with no references
                    if let Some(reference) = textures.iter().find(|(_key, value)| value.reference().is_some()) { return Err(MeshBakeError::UnresolvedTextureReference(format!("key: {} value: {:?}", reference.0, reference.1))) }

                    let uv_map = block_atlas.uv_map.read();

                    let unallocated_textures: Vec<ResourcePath> = textures.iter()
                        .filter_map(|(_, texture)| {
                            let texture_id: ResourcePath = (&texture.0).into();
                            if !uv_map.contains_key(&texture_id) {
                                //Block UV atlas doesn't contain a texture, so we add it
                                Some(texture_id)
                            } else {
                                None
                            }
                        }).collect();

                    drop(uv_map);

                    let unallocated_textures: Vec<(&ResourcePath, Vec<u8>)> = unallocated_textures
                        .iter()
                        .map(|path| (path, resource_provider.get_bytes(&path.prepend("textures/").append(".png")).unwrap()))
                        .collect();

                    if !unallocated_textures.is_empty() {
                        block_atlas.allocate(
                            unallocated_textures.iter()
                                .map(|(path, data)| (*path, data)),
                            resource_provider,
                        );
                    }
                };

                let matrix =
                    Matrix4::from_translation(Vector3::new(0.5, 0.5, 0.5))
                    * Matrix4::from_angle_y(Deg(model_properties.y as f32))
                    * Matrix4::from_translation(Vector3::new(-0.5, -0.5, -0.5));

                let is_cube = model.elements.iter().len() == 1 && {
                    match model.elements.iter().flatten().next() {
                        Some(first) => {
                            first.from[0] == 0.0
                            && first.from[1] == 0.0
                            && first.from[2] == 0.0
                            && first.to[0] == 16.0
                            && first.to[1] == 16.0
                            && first.to[2] == 16.0
                        },
                        None => false,
                    }
                };

                let mut results = model
                    .elements
                    .iter()
                    .flatten()
                    .map(|element| {
                        //Face textures

                        let north = element.faces.get(&schemas::models::BlockFace::North).as_ref().and_then(|tex|
                            get_atlas_uv(
                                tex,
                                block_atlas,
                            ).map(|uv| (
                                //The default UV for this texture
                                uv,
                                //If this texture has an animation, get the offset, otherwise default to 0
                                *block_atlas.animated_texture_offsets.read()
                                    .get(&(&tex.texture.0).into())
                                    .unwrap_or(&0)
                            ))
                        );

                        let east = element.faces.get(&schemas::models::BlockFace::East).as_ref().and_then(|tex|
                            get_atlas_uv(
                                tex,
                                block_atlas,
                            ).map(|uv| (
                                //The default UV for this texture
                                uv,
                                //If this texture has an animation, get the offset, otherwise default to 0
                                *block_atlas.animated_texture_offsets.read()
                                    .get(&(&tex.texture.0).into())
                                    .unwrap_or(&0)
                            ))
                        );

                        let south = element.faces.get(&schemas::models::BlockFace::South).as_ref().and_then(|tex|
                            get_atlas_uv(
                                tex,
                                block_atlas,
                            ).map(|uv| (
                                //The default UV for this texture
                                uv,
                                //If this texture has an animation, get the offset, otherwise default to 0
                                *block_atlas.animated_texture_offsets.read()
                                    .get(&(&tex.texture.0).into())
                                    .unwrap_or(&0)
                            ))
                        );

                        let west = element.faces.get(&schemas::models::BlockFace::West).as_ref().and_then(|tex|
                            get_atlas_uv(
                                tex,
                                block_atlas,
                            ).map(|uv| (
                                //The default UV for this texture
                                uv,
                                //If this texture has an animation, get the offset, otherwise default to 0
                                *block_atlas.animated_texture_offsets.read()
                                    .get(&(&tex.texture.0).into())
                                    .unwrap_or(&0)
                            ))
                        );

                        let up = element.faces.get(&schemas::models::BlockFace::Up).as_ref().and_then(|tex|
                            get_atlas_uv(
                                tex,
                                block_atlas,
                            ).map(|uv| (
                                //The default UV for this texture
                                uv,
                                //If this texture has an animation, get the offset, otherwise default to 0
                                *block_atlas.animated_texture_offsets.read()
                                    .get(&(&tex.texture.0).into())
                                    .unwrap_or(&0)
                            ))
                        );

                        let down = element.faces.get(&schemas::models::BlockFace::Down).as_ref().and_then(|tex|
                            get_atlas_uv(
                                tex,
                                block_atlas,
                            ).map(|uv| (
                                //The default UV for this texture
                                uv,
                                //If this texture has an animation, get the offset, otherwise default to 0
                                *block_atlas.animated_texture_offsets.read()
                                    .get(&(&tex.texture.0).into())
                                    .unwrap_or(&0)
                            ))
                        );

                        let a = (matrix * Vector4::new(1.0 - element.from[0] / 16.0, element.from[1] / 16.0, element.from[2] / 16.0, 1.0)).truncate().into();
                        let b = (matrix * Vector4::new(1.0 - element.to[0] / 16.0, element.from[1] / 16.0, element.from[2] / 16.0, 1.0)).truncate().into();
                        let c = (matrix * Vector4::new(1.0 - element.to[0] / 16.0, element.to[1] / 16.0, element.from[2] / 16.0, 1.0)).truncate().into();
                        let d = (matrix * Vector4::new(1.0 - element.from[0] / 16.0, element.to[1] / 16.0, element.from[2] / 16.0, 1.0)).truncate().into();
                        let e = (matrix * Vector4::new(1.0 - element.from[0] / 16.0, element.from[1] / 16.0, element.to[2] / 16.0, 1.0)).truncate().into();
                        let f = (matrix * Vector4::new(1.0 - element.to[0] / 16.0, element.from[1] / 16.0, element.to[2] / 16.0, 1.0)).truncate().into();
                        let g = (matrix * Vector4::new(1.0 - element.to[0] / 16.0, element.to[1] / 16.0, element.to[2] / 16.0, 1.0)).truncate().into();
                        let h = (matrix * Vector4::new(1.0 - element.from[0] / 16.0, element.to[1] / 16.0, element.to[2] / 16.0, 1.0)).truncate().into();

                        #[rustfmt::skip]
                        let faces = BlockModelFaces {
                            south: south.map(|south| {[
                                BlockMeshVertex { position: e, tex_coords: [south.0.1.0, south.0.1.1], normal: [0.0, 0.0, 1.0, 1.0], animation_uv_offset: south.1 },
                                BlockMeshVertex { position: h, tex_coords: [south.0.1.0, south.0.0.1], normal: [0.0, 0.0, 1.0, 1.0], animation_uv_offset: south.1 },
                                BlockMeshVertex { position: f, tex_coords: [south.0.0.0, south.0.1.1], normal: [0.0, 0.0, 1.0, 1.0], animation_uv_offset: south.1 },
                                BlockMeshVertex { position: h, tex_coords: [south.0.1.0, south.0.0.1], normal: [0.0, 0.0, 1.0, 1.0], animation_uv_offset: south.1 },
                                BlockMeshVertex { position: g, tex_coords: [south.0.0.0, south.0.0.1], normal: [0.0, 0.0, 1.0, 1.0], animation_uv_offset: south.1 },
                                BlockMeshVertex { position: f, tex_coords: [south.0.0.0, south.0.1.1], normal: [0.0, 0.0, 1.0, 1.0], animation_uv_offset: south.1 },
                            ]}),
                            west: west.map(|west| {[
                                BlockMeshVertex { position: g, tex_coords: [west.0.1.0, west.0.0.1], normal: [-1.0, 0.0, 0.0, 1.0], animation_uv_offset: west.1 },
                                BlockMeshVertex { position: b, tex_coords: [west.0.0.0, west.0.1.1], normal: [-1.0, 0.0, 0.0, 1.0], animation_uv_offset: west.1 },
                                BlockMeshVertex { position: f, tex_coords: [west.0.1.0, west.0.1.1], normal: [-1.0, 0.0, 0.0, 1.0], animation_uv_offset: west.1 },
                                BlockMeshVertex { position: c, tex_coords: [west.0.0.0, west.0.0.1], normal: [-1.0, 0.0, 0.0, 1.0], animation_uv_offset: west.1 },
                                BlockMeshVertex { position: b, tex_coords: [west.0.0.0, west.0.1.1], normal: [-1.0, 0.0, 0.0, 1.0], animation_uv_offset: west.1 },
                                BlockMeshVertex { position: g, tex_coords: [west.0.1.0, west.0.0.1], normal: [-1.0, 0.0, 0.0, 1.0], animation_uv_offset: west.1 },
                            ]}),
                            north: north.map(|north| {[
                                BlockMeshVertex { position: c, tex_coords: [north.0.1.0, north.0.0.1], normal: [0.0, 0.0, -1.0, 1.0], animation_uv_offset: north.1 },
                                BlockMeshVertex { position: a, tex_coords: [north.0.0.0, north.0.1.1], normal: [0.0, 0.0, -1.0, 1.0], animation_uv_offset: north.1 },
                                BlockMeshVertex { position: b, tex_coords: [north.0.1.0, north.0.1.1], normal: [0.0, 0.0, -1.0, 1.0], animation_uv_offset: north.1 },
                                BlockMeshVertex { position: d, tex_coords: [north.0.0.0, north.0.0.1], normal: [0.0, 0.0, -1.0, 1.0], animation_uv_offset: north.1 },
                                BlockMeshVertex { position: a, tex_coords: [north.0.0.0, north.0.1.1], normal: [0.0, 0.0, -1.0, 1.0], animation_uv_offset: north.1 },
                                BlockMeshVertex { position: c, tex_coords: [north.0.1.0, north.0.0.1], normal: [0.0, 0.0, -1.0, 1.0], animation_uv_offset: north.1 },
                            ]}),
                            east: east.map(|east| {[
                                BlockMeshVertex { position: e, tex_coords: [east.0.0.0, east.0.1.1], normal: [1.0, 0.0, 0.0, 1.0], animation_uv_offset: east.1 },
                                BlockMeshVertex { position: a, tex_coords: [east.0.1.0, east.0.1.1], normal: [1.0, 0.0, 0.0, 1.0], animation_uv_offset: east.1 },
                                BlockMeshVertex { position: d, tex_coords: [east.0.1.0, east.0.0.1], normal: [1.0, 0.0, 0.0, 1.0], animation_uv_offset: east.1 },
                                BlockMeshVertex { position: d, tex_coords: [east.0.1.0, east.0.0.1], normal: [1.0, 0.0, 0.0, 1.0], animation_uv_offset: east.1 },
                                BlockMeshVertex { position: h, tex_coords: [east.0.0.0, east.0.0.1], normal: [1.0, 0.0, 0.0, 1.0], animation_uv_offset: east.1 },
                                BlockMeshVertex { position: e, tex_coords: [east.0.0.0, east.0.1.1], normal: [1.0, 0.0, 0.0, 1.0], animation_uv_offset: east.1 },
                            ]}),
                            up: up.map(|up| {[
                                BlockMeshVertex { position: g, tex_coords: [up.0.1.0, up.0.0.1], normal: [0.0, 1.0, 0.0, 1.0], animation_uv_offset: up.1 },
                                BlockMeshVertex { position: h, tex_coords: [up.0.0.0, up.0.0.1], normal: [0.0, 1.0, 0.0, 1.0], animation_uv_offset: up.1 },
                                BlockMeshVertex { position: d, tex_coords: [up.0.0.0, up.0.1.1], normal: [0.0, 1.0, 0.0, 1.0], animation_uv_offset: up.1 },
                                BlockMeshVertex { position: c, tex_coords: [up.0.1.0, up.0.1.1], normal: [0.0, 1.0, 0.0, 1.0], animation_uv_offset: up.1 },
                                BlockMeshVertex { position: g, tex_coords: [up.0.1.0, up.0.0.1], normal: [0.0, 1.0, 0.0, 1.0], animation_uv_offset: up.1 },
                                BlockMeshVertex { position: d, tex_coords: [up.0.0.0, up.0.1.1], normal: [0.0, 1.0, 0.0, 1.0], animation_uv_offset: up.1 },
                            ]}),
                            down: down.map(|down| {[
                                BlockMeshVertex { position: f, tex_coords: [down.0.0.0, down.0.1.1], normal: [0.0, -1.0, 0.0, 1.0], animation_uv_offset: down.1 },
                                BlockMeshVertex { position: b, tex_coords: [down.0.0.0, down.0.0.1], normal: [0.0, -1.0, 0.0, 1.0], animation_uv_offset: down.1 },
                                BlockMeshVertex { position: a, tex_coords: [down.0.1.0, down.0.0.1], normal: [0.0, -1.0, 0.0, 1.0], animation_uv_offset: down.1 },
                                BlockMeshVertex { position: f, tex_coords: [down.0.0.0, down.0.1.1], normal: [0.0, -1.0, 0.0, 1.0], animation_uv_offset: down.1 },
                                BlockMeshVertex { position: a, tex_coords: [down.0.1.0, down.0.0.1], normal: [0.0, -1.0, 0.0, 1.0], animation_uv_offset: down.1 },
                                BlockMeshVertex { position: e, tex_coords: [down.0.1.0, down.0.1.1], normal: [0.0, -1.0, 0.0, 1.0], animation_uv_offset: down.1 },
                            ]}),
                        };

                        Ok(faces)
                    }).collect::<Result<Vec<BlockModelFaces>, MeshBakeError>>()?;

                //TODO
                let has_transparency = false;

                Ok((if is_cube {
                        CubeOrComplexMesh::Cube(Box::new(results.pop().unwrap()))
                    } else {
                        CubeOrComplexMesh::Complex(results)
                    },
                    !is_cube || has_transparency
                ))
            }).collect::<Result<Vec<_>, MeshBakeError>>()?;

        Ok(Self { models })
    }
}
