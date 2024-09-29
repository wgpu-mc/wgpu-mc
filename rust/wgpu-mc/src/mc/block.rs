use crate::mc::chunk::RenderLayer;
use glam::{vec3, Mat3, Vec3};
use itertools::Itertools;
use minecraft_assets::api::ModelResolver;
use minecraft_assets::schemas;
use minecraft_assets::schemas::blockstates::ModelProperties;
use serde_derive::{Deserialize, Serialize};

use crate::mc::direction::Direction;
use crate::mc::resource::{ResourcePath, ResourceProvider};
use crate::render::atlas::Atlas;
use crate::texture::UV;

/// A block position: x, y, z
pub type BlockPos = (i32, u16, i32);

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct BlockstateKey {
    /// An index into [BlockManager]
    pub block: u16,
    /// Used to quickly figure out which [ModelMesh] a [Block] should return without having to hash strings
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
#[derive(Copy, Clone, Debug)]
pub struct BlockMeshVertex {
    pub position: Vec3,
    pub tex_coords: [u16; 2],
}
#[derive(Debug, Clone, Copy)]
pub struct BlockModelFace {
    pub vertices: [BlockMeshVertex; 4],
    pub normal: Vec3,
    pub tint_index: i32,
    pub animation_uv_offset: u32,
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
    let uv = face.uv.unwrap_or([0.0, 0.0, 16.0, 16.0]).map(|x| x as u16);
    let atlas_map = block_atlas.uv_map.read();
    atlas_map
        .get(&(&face.texture.0).into())
        .copied()
        .map(|tex| {
            let tw = (tex.1 .0 - tex.0 .0, tex.1 .1 - tex.0 .1);
            let uvs = match face.rotation {
                0 => ((uv[0], uv[1]), (uv[2], uv[3])),
                90 => ((tw.1 - uv[1], uv[0]), (tw.1 - uv[3], uv[2])),
                180 => ((tw.0 - uv[0], tw.1 - uv[1]), (tw.0 - uv[2], tw.1 - uv[3])),
                270 => ((uv[1], tw.0 - uv[0]), (uv[3], tw.0 - uv[2])),
                _ => unreachable!(),
            };
            (
                (tex.0 .0 + uvs.0 .0, tex.0 .1 + uvs.0 .1),
                (tex.0 .0 + uvs.1 .0, tex.0 .1 + uvs.1 .1),
            )
        })
}

pub struct RenderSettings {
    pub opaque: bool,
}

/// TODO: Use actual error handling library
#[derive(Debug)]
pub enum MeshBakeError {
    UnresolvedTextureReference(String),
    UnresolvedResourcePath(ResourcePath),
    JsonError(serde_json::Error),
}

/// A block model which has been baked into a mesh and is ready for rendering
#[derive(Debug)]
pub struct ModelMesh {
    pub north: Vec<BlockModelFace>,
    pub south: Vec<BlockModelFace>,
    pub west: Vec<BlockModelFace>,
    pub east: Vec<BlockModelFace>,
    pub up: Vec<BlockModelFace>,
    pub down: Vec<BlockModelFace>,
    pub any: Vec<BlockModelFace>,
    pub cull: u8,
    pub layer: RenderLayer,
}

impl ModelMesh {
    pub fn bake<'a>(
        model_properties: impl IntoIterator<Item = &'a ModelProperties>,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas,
    ) -> Result<Self, MeshBakeError> {
        let mesh = model_properties
            .into_iter()
            .map(|model_properties: &ModelProperties| {
                let model_resource_path = ResourcePath::from(&model_properties.model)
                    .prepend("models/")
                    .append(".json");

                //Recursively resolve the model using it's parents if it has any
                let model: schemas::Model = resolve_model(
                    //Parse the JSON into the model schema
                    serde_json::from_str(
                        //Get the model JSON
                        &resource_provider
                            .get_string(&model_resource_path)
                            .ok_or_else(|| {
                                MeshBakeError::UnresolvedResourcePath(model_resource_path)
                            })?,
                    )
                    .map_err(MeshBakeError::JsonError)?,
                    resource_provider,
                );
                if let Some(textures) = model.textures {
                    //Make sure the textures in the model are fully resolved with no references
                    if let Some(reference) = textures
                        .iter()
                        .find(|(_key, value)| value.reference().is_some())
                    {
                        return Err(MeshBakeError::UnresolvedTextureReference(format!(
                            "key: {} value: {:?}",
                            reference.0, reference.1
                        )));
                    }

                    let uv_map = block_atlas.uv_map.read();

                    let unallocated_textures: Vec<ResourcePath> = textures
                        .iter()
                        .filter_map(|(_, texture)| {
                            let texture_id: ResourcePath = (&texture.0).into();
                            if !uv_map.contains_key(&texture_id) {
                                //Block UV atlas doesn't contain a texture, so we add it
                                Some(texture_id)
                            } else {
                                None
                            }
                        })
                        .collect();

                    drop(uv_map);

                    let unallocated_textures: Vec<(&ResourcePath, Vec<u8>)> = unallocated_textures
                        .iter()
                        .map(|path| {
                            (
                                path,
                                resource_provider
                                    .get_bytes(&path.prepend("textures/").append(".png"))
                                    .unwrap(),
                            )
                        })
                        .collect();

                    if !unallocated_textures.is_empty() {
                        block_atlas.allocate(
                            unallocated_textures
                                .iter()
                                .map(|(path, data)| (*path, data)),
                            resource_provider,
                        );
                    }
                };

                Ok(model
                    .elements
                    .iter()
                    .flatten()
                    .flat_map(|element| {
                        //Face textures
                        let north = element
                            .faces
                            .get(&schemas::models::BlockFace::North)
                            .as_ref()
                            .and_then(|tex| {
                                get_atlas_uv(tex, block_atlas).map(|uv| {
                                    (
                                        //The default UV for this texture
                                        uv,
                                        //If this texture has an animation, get the offset, otherwise default to 0
                                        *block_atlas
                                            .animated_texture_offsets
                                            .read()
                                            .get(&(&tex.texture.0).into())
                                            .unwrap_or(&0),
                                        tex.tint_index,
                                    )
                                })
                            });

                        let east = element
                            .faces
                            .get(&schemas::models::BlockFace::East)
                            .as_ref()
                            .and_then(|tex| {
                                get_atlas_uv(tex, block_atlas).map(|uv| {
                                    (
                                        //The default UV for this texture
                                        uv,
                                        //If this texture has an animation, get the offset, otherwise default to 0
                                        *block_atlas
                                            .animated_texture_offsets
                                            .read()
                                            .get(&(&tex.texture.0).into())
                                            .unwrap_or(&0),
                                        tex.tint_index,
                                    )
                                })
                            });

                        let south = element
                            .faces
                            .get(&schemas::models::BlockFace::South)
                            .as_ref()
                            .and_then(|tex| {
                                get_atlas_uv(tex, block_atlas).map(|uv| {
                                    (
                                        //The default UV for this texture
                                        uv,
                                        //If this texture has an animation, get the offset, otherwise default to 0
                                        *block_atlas
                                            .animated_texture_offsets
                                            .read()
                                            .get(&(&tex.texture.0).into())
                                            .unwrap_or(&0),
                                        tex.tint_index,
                                    )
                                })
                            });

                        let west = element
                            .faces
                            .get(&schemas::models::BlockFace::West)
                            .as_ref()
                            .and_then(|tex| {
                                get_atlas_uv(tex, block_atlas).map(|uv| {
                                    (
                                        //The default UV for this texture
                                        uv,
                                        //If this texture has an animation, get the offset, otherwise default to 0
                                        *block_atlas
                                            .animated_texture_offsets
                                            .read()
                                            .get(&(&tex.texture.0).into())
                                            .unwrap_or(&0),
                                        tex.tint_index,
                                    )
                                })
                            });

                        let up = element
                            .faces
                            .get(&schemas::models::BlockFace::Up)
                            .as_ref()
                            .and_then(|tex| {
                                get_atlas_uv(tex, block_atlas).map(|uv| {
                                    (
                                        //The default UV for this texture
                                        uv,
                                        //If this texture has an animation, get the offset, otherwise default to 0
                                        *block_atlas
                                            .animated_texture_offsets
                                            .read()
                                            .get(&(&tex.texture.0).into())
                                            .unwrap_or(&0),
                                        tex.tint_index,
                                    )
                                })
                            });

                        let down = element
                            .faces
                            .get(&schemas::models::BlockFace::Down)
                            .as_ref()
                            .and_then(|tex| {
                                get_atlas_uv(tex, block_atlas).map(|uv| {
                                    (
                                        //The default UV for this texture
                                        uv,
                                        //If this texture has an animation, get the offset, otherwise default to 0
                                        *block_atlas
                                            .animated_texture_offsets
                                            .read()
                                            .get(&(&tex.texture.0).into())
                                            .unwrap_or(&0),
                                        tex.tint_index,
                                    )
                                })
                            });
                        let rot = &element.rotation;
                        let matrix = match rot.axis {
                            schemas::models::Axis::X => {
                                Mat3::from_rotation_x(rot.angle.to_radians())
                            }
                            schemas::models::Axis::Y => {
                                Mat3::from_rotation_y(rot.angle.to_radians())
                            }
                            schemas::models::Axis::Z => {
                                Mat3::from_rotation_z(rot.angle.to_radians())
                            }
                        };
                        let vec_origin = Vec3::from_array(rot.origin) / 16.0;

                        let vertex_transform = |v: Vec3| {
                            let v = match model_properties.x {
                                0 => v,
                                90 => vec3(v.x, 1.0 - v.z, v.y),
                                180 => vec3(v.x, 1.0 - v.y, 1.0 - v.z),
                                270 => vec3(v.x, v.z, 1.0 - v.y),
                                _ => panic!("invalid rotation"),
                            };
                            let v = matrix * (v - vec_origin) + vec_origin;

                            match model_properties.y {
                                0 => v,
                                90 => vec3(1.0 - v.z, v.y, v.x),
                                180 => vec3(1.0 - v.x, v.y, 1.0 - v.z),
                                270 => vec3(v.z, v.y, 1.0 - v.x),
                                _ => panic!("invalid rotation"),
                            }
                        };

                        let p000 = vertex_transform(vec3(
                            element.from[0] / 16.0,
                            element.from[1] / 16.0,
                            element.from[2] / 16.0,
                        ));
                        let p001 = vertex_transform(vec3(
                            element.from[0] / 16.0,
                            element.from[1] / 16.0,
                            element.to[2] / 16.0,
                        ));
                        let p010 = vertex_transform(vec3(
                            element.from[0] / 16.0,
                            element.to[1] / 16.0,
                            element.from[2] / 16.0,
                        ));
                        let p011 = vertex_transform(vec3(
                            element.from[0] / 16.0,
                            element.to[1] / 16.0,
                            element.to[2] / 16.0,
                        ));
                        let p100 = vertex_transform(vec3(
                            element.to[0] / 16.0,
                            element.from[1] / 16.0,
                            element.from[2] / 16.0,
                        ));
                        let p101 = vertex_transform(vec3(
                            element.to[0] / 16.0,
                            element.from[1] / 16.0,
                            element.to[2] / 16.0,
                        ));
                        let p110 = vertex_transform(vec3(
                            element.to[0] / 16.0,
                            element.to[1] / 16.0,
                            element.from[2] / 16.0,
                        ));
                        let p111 = vertex_transform(vec3(
                            element.to[0] / 16.0,
                            element.to[1] / 16.0,
                            element.to[2] / 16.0,
                        ));

                        let mut faces = vec![];
                        faces.extend(south.map(|south_face| BlockModelFace {
                            vertices: [
                                BlockMeshVertex {
                                    position: p101,
                                    tex_coords: [south_face.0 .1 .0, south_face.0 .1 .1],
                                },
                                BlockMeshVertex {
                                    position: p111,
                                    tex_coords: [south_face.0 .1 .0, south_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p011,
                                    tex_coords: [south_face.0 .0 .0, south_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p001,
                                    tex_coords: [south_face.0 .0 .0, south_face.0 .1 .1],
                                },
                            ],
                            normal: vec3(0.0, 0.0, 1.0),
                            tint_index: south_face.2,
                            animation_uv_offset: south_face.1,
                        }));
                        faces.extend(west.map(|west_face| BlockModelFace {
                            vertices: [
                                BlockMeshVertex {
                                    position: p001,
                                    tex_coords: [west_face.0 .1 .0, west_face.0 .1 .1],
                                },
                                BlockMeshVertex {
                                    position: p011,
                                    tex_coords: [west_face.0 .1 .0, west_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p010,
                                    tex_coords: [west_face.0 .0 .0, west_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p000,
                                    tex_coords: [west_face.0 .0 .0, west_face.0 .1 .1],
                                },
                            ],
                            normal: vec3(-1.0, 0.0, 0.0),
                            tint_index: west_face.2,
                            animation_uv_offset: west_face.1,
                        }));
                        faces.extend(north.map(|north_face| BlockModelFace {
                            vertices: [
                                BlockMeshVertex {
                                    position: p000,
                                    tex_coords: [north_face.0 .1 .0, north_face.0 .1 .1],
                                },
                                BlockMeshVertex {
                                    position: p010,
                                    tex_coords: [north_face.0 .1 .0, north_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p110,
                                    tex_coords: [north_face.0 .0 .0, north_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p100,
                                    tex_coords: [north_face.0 .0 .0, north_face.0 .1 .1],
                                },
                            ],
                            normal: vec3(0.0, 0.0, -1.0),
                            tint_index: north_face.2,
                            animation_uv_offset: north_face.1,
                        }));
                        faces.extend(east.map(|east_face| BlockModelFace {
                            vertices: [
                                BlockMeshVertex {
                                    position: p100,
                                    tex_coords: [east_face.0 .1 .0, east_face.0 .1 .1],
                                },
                                BlockMeshVertex {
                                    position: p110,
                                    tex_coords: [east_face.0 .1 .0, east_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p111,
                                    tex_coords: [east_face.0 .0 .0, east_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p101,
                                    tex_coords: [east_face.0 .0 .0, east_face.0 .1 .1],
                                },
                            ],
                            normal: vec3(1.0, 0.0, 0.0),
                            tint_index: east_face.2,
                            animation_uv_offset: east_face.1,
                        }));
                        faces.extend(up.map(|up_face| BlockModelFace {
                            vertices: [
                                BlockMeshVertex {
                                    position: p010,
                                    tex_coords: [up_face.0 .1 .0, up_face.0 .1 .1],
                                },
                                BlockMeshVertex {
                                    position: p011,
                                    tex_coords: [up_face.0 .1 .0, up_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p111,
                                    tex_coords: [up_face.0 .0 .0, up_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p110,
                                    tex_coords: [up_face.0 .0 .0, up_face.0 .1 .1],
                                },
                            ],
                            normal: vec3(0.0, 1.0, 0.0),
                            tint_index: up_face.2,
                            animation_uv_offset: up_face.1,
                        }));

                        faces.extend(down.map(|down_face| BlockModelFace {
                            vertices: [
                                BlockMeshVertex {
                                    position: p000,
                                    tex_coords: [down_face.0 .1 .0, down_face.0 .1 .1],
                                },
                                BlockMeshVertex {
                                    position: p100,
                                    tex_coords: [down_face.0 .1 .0, down_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p101,
                                    tex_coords: [down_face.0 .0 .0, down_face.0 .0 .1],
                                },
                                BlockMeshVertex {
                                    position: p001,
                                    tex_coords: [down_face.0 .0 .0, down_face.0 .1 .1],
                                },
                            ],
                            normal: vec3(0.0, -1.0, 0.0),
                            tint_index: down_face.2,
                            animation_uv_offset: down_face.1,
                        }));
                        faces
                    })
                    .collect::<Vec<BlockModelFace>>())
            })
            .flatten_ok()
            .collect::<Result<Vec<BlockModelFace>, MeshBakeError>>()?;
        let mut result = Self {
            layer: RenderLayer::Solid,
            north: vec![],
            south: vec![],
            west: vec![],
            east: vec![],
            up: vec![],
            down: vec![],
            any: vec![],
            cull: 0,
        };
        mesh.iter().for_each(|face| {
            let full_face = (face.vertices[0].position.fract() == vec3(0.0, 0.0, 0.0)
                && face.vertices[1].position.fract() == vec3(0.0, 0.0, 0.0)
                && face.vertices[2].position.fract() == vec3(0.0, 0.0, 0.0)
                && face.vertices[3].position.fract() == vec3(0.0, 0.0, 0.0))
                as u8;
            if face.vertices[0].position.x == 0.0
                && face.vertices[1].position.x == 0.0
                && face.vertices[2].position.x == 0.0
            {
                result.west.push(*face);
                result.cull |= full_face << Direction::West as u8;
            } else if face.vertices[0].position.x == 1.0
                && face.vertices[1].position.x == 1.0
                && face.vertices[2].position.x == 1.0
            {
                result.east.push(*face);
                result.cull |= full_face << Direction::East as u8;
            } else if face.vertices[0].position.y == 0.0
                && face.vertices[1].position.y == 0.0
                && face.vertices[2].position.y == 0.0
            {
                result.down.push(*face);
                result.cull |= full_face << Direction::Down as u8;
            } else if face.vertices[0].position.y == 1.0
                && face.vertices[1].position.y == 1.0
                && face.vertices[2].position.y == 1.0
            {
                result.up.push(*face);
                result.cull |= full_face << Direction::Up as u8;
            } else if face.vertices[0].position.z == 0.0
                && face.vertices[1].position.z == 0.0
                && face.vertices[2].position.z == 0.0
            {
                result.north.push(*face);
                result.cull |= full_face << Direction::North as u8;
            } else if face.vertices[0].position.z == 1.0
                && face.vertices[1].position.z == 1.0
                && face.vertices[2].position.z == 1.0
            {
                result.south.push(*face);
                result.cull |= full_face << Direction::South as u8;
            } else {
                result.any.push(*face);
            }
        });
        Ok(result)
    }
}
