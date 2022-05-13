use std::collections::HashMap;

use crate::mc::block::blockstate::BlockstateVariantModelDefinitionRotations;
use crate::mc::datapack;
use crate::mc::datapack::{FaceTexture, NamespacedResource, TextureVariableOrResource};
use crate::mc::resource::ResourceProvider;
use crate::model::{AnimatedMeshVertex, MeshVertex};
use crate::render::atlas::{TextureManager, ATLAS_DIMENSIONS};
use crate::render::pipeline::terrain::BLOCK_ATLAS_NAME;
use crate::texture::UV;
use cgmath::{Matrix3, SquareMatrix, Vector3};

#[derive(Debug)]
pub struct BlockModelFaces {
    pub north: Option<[AnimatedMeshVertex; 6]>,
    pub east: Option<[AnimatedMeshVertex; 6]>,
    pub south: Option<[AnimatedMeshVertex; 6]>,
    pub west: Option<[AnimatedMeshVertex; 6]>,
    pub up: Option<[AnimatedMeshVertex; 6]>,
    pub down: Option<[AnimatedMeshVertex; 6]>,
}

#[derive(Debug)]
///Makes chunk mesh baking a bit faster
pub enum CubeOrComplexMesh {
    Cube(Box<BlockModelFaces>),
    Custom(Vec<BlockModelFaces>),
}

#[derive(Debug)]
pub struct BlockstateVariantMesh {
    pub name: NamespacedResource,
    pub shape: CubeOrComplexMesh,
    pub transparent_or_complex: bool,
}

impl BlockstateVariantMesh {
    pub fn absolute_atlas_uv(
        face: &FaceTexture,
        tex_manager: &TextureManager,
        textures: &HashMap<String, TextureVariableOrResource>,
    ) -> Option<UV> {
        let block_atlas = tex_manager
            .atlases
            .load()
            .get(BLOCK_ATLAS_NAME)
            .unwrap()
            .load_full();

        let atlas_map = block_atlas.uv_map.read();

        let face_resource = face.texture.recurse_resolve_as_resource(textures)?;

        let atlas_uv = atlas_map.get(face_resource).unwrap_or_else(|| {
            panic!(
                "{:?}\n{:?} {} {}",
                block_atlas,
                face_resource,
                face_resource.0.len(),
                face_resource.1.len()
            )
        });

        let _face_uv = &face.uv;

        const ATLAS: f32 = ATLAS_DIMENSIONS as f32;

        let adjusted_uv = (
            ((atlas_uv.0 .0) / ATLAS, (atlas_uv.0 .1) / ATLAS),
            ((atlas_uv.1 .0) / ATLAS, (atlas_uv.1 .1) / ATLAS),
        );

        Some(adjusted_uv)
    }

    pub fn bake_block_model(
        model: &datapack::BlockModel,
        _rp: &dyn ResourceProvider,
        tex_manager: &TextureManager,
        _transform: &BlockstateVariantModelDefinitionRotations,
    ) -> Option<Self> {
        let _texture_ids = &model.textures;

        // let matrix = Matrix3::from(
        //     Euler {
        //         x: Deg(transform.x as f32),
        //         y: Deg(transform.y as f32),
        //         z: Deg(transform.z as f32)
        //     }
        // );
        let matrix = Matrix3::identity();

        let is_cube = model.elements.len() == 1 && {
            let first = model.elements.first().unwrap();

            first.from.0 == 0.0
                && first.from.1 == 0.0
                && first.from.2 == 0.0
                && first.to.0 == 1.0
                && first.to.1 == 1.0
                && first.to.2 == 1.0
        };

        let mut results = model
            .elements
            .iter()
            .map(|element| {
                //Face textures

                // println!("{:?}", element);
                let north = element.face_textures.north.as_ref().and_then(|tex|
                    match Self::absolute_atlas_uv(
                    tex,
                    tex_manager,
                    &model.textures
                    ) {
                        None => None,
                        Some(f) => {
                            match tex_manager.atlases
                                .load_full()
                                .get(BLOCK_ATLAS_NAME)
                                .unwrap().load_full().animated_texture_offsets.load_full()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures).unwrap()) {
                                None => Some((f, 0 as u32)),
                                Some(t) => Some((f, *t)),
                            }
                        }
                    }
                );

                let east = element.face_textures.east.as_ref().and_then(|tex| {
                    match Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ) {
                        None => None,
                        Some(f) => {
                            match tex_manager.atlases
                                .load_full()
                                .get(BLOCK_ATLAS_NAME)
                                .unwrap().load_full().animated_texture_offsets.load_full()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures).unwrap()) {
                                None => Some((f, 0 as u32)),
                                Some(t) => Some((f, *t)),
                            }
                        }
                    }
                });

                let south = element.face_textures.south.as_ref().and_then(|tex| {
                    match Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ) {
                        None => None,
                        Some(f) => {
                            match tex_manager.atlases
                                .load_full()
                                .get(BLOCK_ATLAS_NAME)
                                .unwrap().load_full().animated_texture_offsets.load_full()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures).unwrap()) {
                                None => Some((f, 0 as u32)),
                                Some(t) => Some((f, *t)),
                            }
                        }
                    }
                });

                let west = element.face_textures.west.as_ref().and_then(|tex| {
                    match Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ) {
                        None => None,
                        Some(f) => {
                            match tex_manager.atlases
                                .load_full()
                                .get(BLOCK_ATLAS_NAME)
                                .unwrap().load_full().animated_texture_offsets.load_full()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures).unwrap()) {
                                None => Some((f, 0 as u32)),
                                Some(t) => Some((f, *t)),
                            }
                        }
                    }
                });

                let up = element.face_textures.up.as_ref().and_then(|tex| {
                    match Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ) {
                        None => None,
                        Some(f) => {
                            match tex_manager.atlases
                                .load_full()
                                .get(BLOCK_ATLAS_NAME)
                                .unwrap().load_full().animated_texture_offsets.load_full()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures).unwrap()) {
                                None => Some((f, 0 as u32)),
                                Some(t) => Some((f, *t)),
                            }
                        }
                    }
                });

                let down = element.face_textures.down.as_ref().and_then(|tex| {
                    match Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ) {
                        None => None,
                        Some(f) => {
                            match tex_manager.atlases
                                .load_full()
                                .get(BLOCK_ATLAS_NAME)
                                .unwrap().load_full().animated_texture_offsets.load_full()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures).unwrap()) {
                                None => Some((f, 0 as u32)),
                                Some(t) => Some((f, *t)),
                            }
                        }
                    }
                });

                let a = (matrix * Vector3::new(1.0 - element.from.0, element.from.1, element.from.2)).into();
                let b = (matrix * Vector3::new(1.0 - element.to.0, element.from.1, element.from.2)).into();
                let c = (matrix * Vector3::new(1.0 - element.to.0, element.to.1, element.from.2)).into();
                let d = (matrix * Vector3::new(1.0 - element.from.0, element.to.1, element.from.2)).into();
                let e = (matrix * Vector3::new(1.0 - element.from.0, element.from.1, element.to.2)).into();
                let f = (matrix * Vector3::new(1.0 - element.to.0, element.from.1, element.to.2)).into();
                let g = (matrix * Vector3::new(1.0 - element.to.0, element.to.1, element.to.2)).into();
                let h = (matrix * Vector3::new(1.0 - element.from.0, element.to.1, element.to.2)).into();

                #[rustfmt::skip]
                let faces = BlockModelFaces {
                    south: south.map(|south| {[
                        AnimatedMeshVertex { position: e, tex_coords: [south.0.1.0, south.0.1.1], normal: [0.0, 0.0, 1.0], uv_offset: south.1 },
                        AnimatedMeshVertex { position: h, tex_coords: [south.0.1.0, south.0.0.1], normal: [0.0, 0.0, 1.0], uv_offset: south.1 },
                        AnimatedMeshVertex { position: f, tex_coords: [south.0.0.0, south.0.1.1], normal: [0.0, 0.0, 1.0], uv_offset: south.1 },
                        AnimatedMeshVertex { position: h, tex_coords: [south.0.1.0, south.0.0.1], normal: [0.0, 0.0, 1.0], uv_offset: south.1 },
                        AnimatedMeshVertex { position: g, tex_coords: [south.0.0.0, south.0.0.1], normal: [0.0, 0.0, 1.0], uv_offset: south.1 },
                        AnimatedMeshVertex { position: f, tex_coords: [south.0.0.0, south.0.1.1], normal: [0.0, 0.0, 1.0], uv_offset: south.1 },
                    ]}),
                    west: west.map(|west| {[
                        AnimatedMeshVertex { position: g, tex_coords: [west.0.1.0, west.0.0.1], normal: [-1.0, 0.0, 0.0], uv_offset: west.1 },
                        AnimatedMeshVertex { position: b, tex_coords: [west.0.0.0, west.0.1.1], normal: [-1.0, 0.0, 0.0], uv_offset: west.1 },
                        AnimatedMeshVertex { position: f, tex_coords: [west.0.1.0, west.0.1.1], normal: [-1.0, 0.0, 0.0], uv_offset: west.1 },
                        AnimatedMeshVertex { position: c, tex_coords: [west.0.0.0, west.0.0.1], normal: [-1.0, 0.0, 0.0], uv_offset: west.1 },
                        AnimatedMeshVertex { position: b, tex_coords: [west.0.0.0, west.0.1.1], normal: [-1.0, 0.0, 0.0], uv_offset: west.1 },
                        AnimatedMeshVertex { position: g, tex_coords: [west.0.1.0, west.0.0.1], normal: [-1.0, 0.0, 0.0], uv_offset: west.1 },
                    ]}),
                    north: north.map(|north| {[
                        AnimatedMeshVertex { position: c, tex_coords: [north.0.1.0, north.0.0.1], normal: [0.0, 0.0, -1.0], uv_offset: north.1 },
                        AnimatedMeshVertex { position: a, tex_coords: [north.0.0.0, north.0.1.1], normal: [0.0, 0.0, -1.0], uv_offset: north.1 },
                        AnimatedMeshVertex { position: b, tex_coords: [north.0.1.0, north.0.1.1], normal: [0.0, 0.0, -1.0], uv_offset: north.1 },
                        AnimatedMeshVertex { position: d, tex_coords: [north.0.0.0, north.0.0.1], normal: [0.0, 0.0, -1.0], uv_offset: north.1 },
                        AnimatedMeshVertex { position: a, tex_coords: [north.0.0.0, north.0.1.1], normal: [0.0, 0.0, -1.0], uv_offset: north.1 },
                        AnimatedMeshVertex { position: c, tex_coords: [north.0.1.0, north.0.0.1], normal: [0.0, 0.0, -1.0], uv_offset: north.1 },
                    ]}),
                    east: east.map(|east| {[
                        AnimatedMeshVertex { position: e, tex_coords: [east.0.0.0, east.0.1.1], normal: [1.0, 0.0, 0.0], uv_offset: east.1 },
                        AnimatedMeshVertex { position: a, tex_coords: [east.0.1.0, east.0.1.1], normal: [1.0, 0.0, 0.0], uv_offset: east.1 },
                        AnimatedMeshVertex { position: d, tex_coords: [east.0.1.0, east.0.0.1], normal: [1.0, 0.0, 0.0], uv_offset: east.1 },
                        AnimatedMeshVertex { position: d, tex_coords: [east.0.1.0, east.0.0.1], normal: [1.0, 0.0, 0.0], uv_offset: east.1 },
                        AnimatedMeshVertex { position: h, tex_coords: [east.0.0.0, east.0.0.1], normal: [1.0, 0.0, 0.0], uv_offset: east.1 },
                        AnimatedMeshVertex { position: e, tex_coords: [east.0.0.0, east.0.1.1], normal: [1.0, 0.0, 0.0], uv_offset: east.1 },
                    ]}),
                    up: up.map(|up| {[
                        AnimatedMeshVertex { position: g, tex_coords: [up.0.1.0, up.0.0.1], normal: [0.0, 1.0, 0.0], uv_offset: up.1 },
                        AnimatedMeshVertex { position: h, tex_coords: [up.0.0.0, up.0.0.1], normal: [0.0, 1.0, 0.0], uv_offset: up.1 },
                        AnimatedMeshVertex { position: d, tex_coords: [up.0.0.0, up.0.1.1], normal: [0.0, 1.0, 0.0], uv_offset: up.1 },
                        AnimatedMeshVertex { position: c, tex_coords: [up.0.1.0, up.0.1.1], normal: [0.0, 1.0, 0.0], uv_offset: up.1 },
                        AnimatedMeshVertex { position: g, tex_coords: [up.0.1.0, up.0.0.1], normal: [0.0, 1.0, 0.0], uv_offset: up.1 },
                        AnimatedMeshVertex { position: d, tex_coords: [up.0.0.0, up.0.1.1], normal: [0.0, 1.0, 0.0], uv_offset: up.1 },
                    ]}),
                    down: down.map(|down| {[
                        AnimatedMeshVertex { position: f, tex_coords: [down.0.0.0, down.0.1.1], normal: [0.0, -1.0, 0.0], uv_offset: down.1 },
                        AnimatedMeshVertex { position: b, tex_coords: [down.0.0.0, down.0.0.1], normal: [0.0, -1.0, 0.0], uv_offset: down.1 },
                        AnimatedMeshVertex { position: a, tex_coords: [down.0.1.0, down.0.0.1], normal: [0.0, -1.0, 0.0], uv_offset: down.1 },
                        AnimatedMeshVertex { position: f, tex_coords: [down.0.0.0, down.0.1.1], normal: [0.0, -1.0, 0.0], uv_offset: down.1 },
                        AnimatedMeshVertex { position: a, tex_coords: [down.0.1.0, down.0.0.1], normal: [0.0, -1.0, 0.0], uv_offset: down.1 },
                        AnimatedMeshVertex { position: e, tex_coords: [down.0.1.0, down.0.1.1], normal: [0.0, -1.0, 0.0], uv_offset: down.1 },
                    ]}),
                };

                Some(faces)
            })
            .collect::<Option<Vec<BlockModelFaces>>>()?;

        //TODO
        let has_transparency = false;

        Some(Self {
            name: model.id.clone(),
            shape: if is_cube {
                CubeOrComplexMesh::Cube(Box::new(results.pop().unwrap()))
            } else {
                CubeOrComplexMesh::Custom(results)
            },
            transparent_or_complex: !is_cube || has_transparency,
        })
    }
}
