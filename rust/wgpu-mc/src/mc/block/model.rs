use std::collections::HashMap;
use std::sync::Arc;

use crate::mc::block::blockstate::BlockModelRotations;
use crate::mc::datapack;
use crate::mc::datapack::{FaceTexture, NamespacedResource, TextureVariableOrResource};
use crate::mc::resource::ResourceProvider;
use crate::model::{BlockMeshVertex};
use crate::render::atlas::{TextureManager, ATLAS_DIMENSIONS};
use crate::render::pipeline::terrain::BLOCK_ATLAS_NAME;
use crate::texture::UV;
use cgmath::{Matrix3, SquareMatrix, Vector3};
use crate::mc::block::Multipart;

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
    Custom(Vec<BlockModelFaces>),
}

///A block can be defined as having simple variants, or it can be multipart, meaning multiple models
/// will be applied when this block is being baked into a chunk, depending on the block's state.
#[derive(Debug)]
pub enum BlockStateDefinitionType {
    Variant(BlockModelMesh),
    Multipart(Arc<Multipart>)
}

///Represents a block that has completely deserialized and is ready to be used in world rendering.
/// This can either be a multipart definition, or representing a single [BlockModelMesh]
#[derive(Debug)]
pub struct BlockVariant {
    pub name: NamespacedResource,
    pub kind: BlockStateDefinitionType,
    ///True if this `self.kind` is [BlockStateDefinitionType::Multipart] and that [BlockModelMesh] also has transparent_or_complex set to true
    pub transparent_or_complex: bool
}

///A block model which has been baked into a mesh and is ready for rendering
#[derive(Debug)]
pub struct BlockModelMesh {
    pub name: NamespacedResource,
    pub shape: CubeOrComplexMesh,
    ///Used as a rendering hint for block side culling
    pub transparent_or_complex: bool,
}

impl BlockModelMesh {
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
        _transform: &BlockModelRotations,
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

                let north = element.face_textures.north.as_ref().and_then(|tex|
                    Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ).and_then(|uv| {
                        Some((
                            //The default UV for this texture
                            uv,
                            //If this texture has an animation, get the offset, otherwise default to 0
                            *tex_manager.atlases
                                .load()
                                .get(BLOCK_ATLAS_NAME)?
                                .load().animated_texture_offsets.load()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures)?)
                                .unwrap_or(&0),
                        ))
                    })
                );

                let east = element.face_textures.east.as_ref().and_then(|tex| {
                    Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ).and_then(|uv| {
                        Some((
                            //The default UV for this texture
                            uv,
                            //If this texture has an animation, get the offset, otherwise default to 0
                            *tex_manager.atlases
                                .load()
                                .get(BLOCK_ATLAS_NAME)?
                                .load().animated_texture_offsets.load()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures)?)
                                .unwrap_or(&0),
                        ))
                    })
                });

                let south = element.face_textures.south.as_ref().and_then(|tex| {
                    Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ).and_then(|uv| {
                        Some((
                            //The default UV for this texture
                            uv,
                            //If this texture has an animation, get the offset, otherwise default to 0
                            *tex_manager.atlases
                                .load()
                                .get(BLOCK_ATLAS_NAME)?
                                .load().animated_texture_offsets.load()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures)?)
                                .unwrap_or(&0),
                        ))
                    })
                });

                let west = element.face_textures.west.as_ref().and_then(|tex| {
                    Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ).and_then(|uv| {
                        Some((
                            //The default UV for this texture
                            uv,
                            //If this texture has an animation, get the offset, otherwise default to 0
                            *tex_manager.atlases
                                .load()
                                .get(BLOCK_ATLAS_NAME)?
                                .load().animated_texture_offsets.load()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures)?)
                                .unwrap_or(&0),
                        ))
                    })
                });

                let up = element.face_textures.up.as_ref().and_then(|tex| {
                    Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ).and_then(|uv| {
                        Some((
                            //The default UV for this texture
                            uv,
                            //If this texture has an animation, get the offset, otherwise default to 0
                            *tex_manager.atlases
                                .load()
                                .get(BLOCK_ATLAS_NAME)?
                                .load().animated_texture_offsets.load()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures)?)
                                .unwrap_or(&0),
                        ))
                    })
                });

                let down = element.face_textures.down.as_ref().and_then(|tex| {
                    Self::absolute_atlas_uv(
                        tex,
                        tex_manager,
                        &model.textures
                    ).and_then(|uv| {
                        Some((
                            //The default UV for this texture
                            uv,
                            //If this texture has an animation, get the offset, otherwise default to 0
                            *tex_manager.atlases
                                .load()
                                .get(BLOCK_ATLAS_NAME)?
                                .load().animated_texture_offsets.load()
                                .get(tex.texture.recurse_resolve_as_resource(&model.textures)?)
                                .unwrap_or(&0),
                        ))
                    })
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
                        BlockMeshVertex { position: e, tex_coords: [south.0.1.0, south.0.1.1], normal: [0.0, 0.0, 1.0, 1.0], uv_offset: south.1 },
                        BlockMeshVertex { position: h, tex_coords: [south.0.1.0, south.0.0.1], normal: [0.0, 0.0, 1.0, 1.0], uv_offset: south.1 },
                        BlockMeshVertex { position: f, tex_coords: [south.0.0.0, south.0.1.1], normal: [0.0, 0.0, 1.0, 1.0], uv_offset: south.1 },
                        BlockMeshVertex { position: h, tex_coords: [south.0.1.0, south.0.0.1], normal: [0.0, 0.0, 1.0, 1.0], uv_offset: south.1 },
                        BlockMeshVertex { position: g, tex_coords: [south.0.0.0, south.0.0.1], normal: [0.0, 0.0, 1.0, 1.0], uv_offset: south.1 },
                        BlockMeshVertex { position: f, tex_coords: [south.0.0.0, south.0.1.1], normal: [0.0, 0.0, 1.0, 1.0], uv_offset: south.1 },
                    ]}),
                    west: west.map(|west| {[
                        BlockMeshVertex { position: g, tex_coords: [west.0.1.0, west.0.0.1], normal: [-1.0, 0.0, 0.0, 1.0], uv_offset: west.1 },
                        BlockMeshVertex { position: b, tex_coords: [west.0.0.0, west.0.1.1], normal: [-1.0, 0.0, 0.0, 1.0], uv_offset: west.1 },
                        BlockMeshVertex { position: f, tex_coords: [west.0.1.0, west.0.1.1], normal: [-1.0, 0.0, 0.0, 1.0], uv_offset: west.1 },
                        BlockMeshVertex { position: c, tex_coords: [west.0.0.0, west.0.0.1], normal: [-1.0, 0.0, 0.0, 1.0], uv_offset: west.1 },
                        BlockMeshVertex { position: b, tex_coords: [west.0.0.0, west.0.1.1], normal: [-1.0, 0.0, 0.0, 1.0], uv_offset: west.1 },
                        BlockMeshVertex { position: g, tex_coords: [west.0.1.0, west.0.0.1], normal: [-1.0, 0.0, 0.0, 1.0], uv_offset: west.1 },
                    ]}),
                    north: north.map(|north| {[
                        BlockMeshVertex { position: c, tex_coords: [north.0.1.0, north.0.0.1], normal: [0.0, 0.0, -1.0, 1.0], uv_offset: north.1 },
                        BlockMeshVertex { position: a, tex_coords: [north.0.0.0, north.0.1.1], normal: [0.0, 0.0, -1.0, 1.0], uv_offset: north.1 },
                        BlockMeshVertex { position: b, tex_coords: [north.0.1.0, north.0.1.1], normal: [0.0, 0.0, -1.0, 1.0], uv_offset: north.1 },
                        BlockMeshVertex { position: d, tex_coords: [north.0.0.0, north.0.0.1], normal: [0.0, 0.0, -1.0, 1.0], uv_offset: north.1 },
                        BlockMeshVertex { position: a, tex_coords: [north.0.0.0, north.0.1.1], normal: [0.0, 0.0, -1.0, 1.0], uv_offset: north.1 },
                        BlockMeshVertex { position: c, tex_coords: [north.0.1.0, north.0.0.1], normal: [0.0, 0.0, -1.0, 1.0], uv_offset: north.1 },
                    ]}),
                    east: east.map(|east| {[
                        BlockMeshVertex { position: e, tex_coords: [east.0.0.0, east.0.1.1], normal: [1.0, 0.0, 0.0, 1.0], uv_offset: east.1 },
                        BlockMeshVertex { position: a, tex_coords: [east.0.1.0, east.0.1.1], normal: [1.0, 0.0, 0.0, 1.0], uv_offset: east.1 },
                        BlockMeshVertex { position: d, tex_coords: [east.0.1.0, east.0.0.1], normal: [1.0, 0.0, 0.0, 1.0], uv_offset: east.1 },
                        BlockMeshVertex { position: d, tex_coords: [east.0.1.0, east.0.0.1], normal: [1.0, 0.0, 0.0, 1.0], uv_offset: east.1 },
                        BlockMeshVertex { position: h, tex_coords: [east.0.0.0, east.0.0.1], normal: [1.0, 0.0, 0.0, 1.0], uv_offset: east.1 },
                        BlockMeshVertex { position: e, tex_coords: [east.0.0.0, east.0.1.1], normal: [1.0, 0.0, 0.0, 1.0], uv_offset: east.1 },
                    ]}),
                    up: up.map(|up| {[
                        BlockMeshVertex { position: g, tex_coords: [up.0.1.0, up.0.0.1], normal: [0.0, 1.0, 0.0, 1.0], uv_offset: up.1 },
                        BlockMeshVertex { position: h, tex_coords: [up.0.0.0, up.0.0.1], normal: [0.0, 1.0, 0.0, 1.0], uv_offset: up.1 },
                        BlockMeshVertex { position: d, tex_coords: [up.0.0.0, up.0.1.1], normal: [0.0, 1.0, 0.0, 1.0], uv_offset: up.1 },
                        BlockMeshVertex { position: c, tex_coords: [up.0.1.0, up.0.1.1], normal: [0.0, 1.0, 0.0, 1.0], uv_offset: up.1 },
                        BlockMeshVertex { position: g, tex_coords: [up.0.1.0, up.0.0.1], normal: [0.0, 1.0, 0.0, 1.0], uv_offset: up.1 },
                        BlockMeshVertex { position: d, tex_coords: [up.0.0.0, up.0.1.1], normal: [0.0, 1.0, 0.0, 1.0], uv_offset: up.1 },
                    ]}),
                    down: down.map(|down| {[
                        BlockMeshVertex { position: f, tex_coords: [down.0.0.0, down.0.1.1], normal: [0.0, -1.0, 0.0, 1.0], uv_offset: down.1 },
                        BlockMeshVertex { position: b, tex_coords: [down.0.0.0, down.0.0.1], normal: [0.0, -1.0, 0.0, 1.0], uv_offset: down.1 },
                        BlockMeshVertex { position: a, tex_coords: [down.0.1.0, down.0.0.1], normal: [0.0, -1.0, 0.0, 1.0], uv_offset: down.1 },
                        BlockMeshVertex { position: f, tex_coords: [down.0.0.0, down.0.1.1], normal: [0.0, -1.0, 0.0, 1.0], uv_offset: down.1 },
                        BlockMeshVertex { position: a, tex_coords: [down.0.1.0, down.0.0.1], normal: [0.0, -1.0, 0.0, 1.0], uv_offset: down.1 },
                        BlockMeshVertex { position: e, tex_coords: [down.0.1.0, down.0.1.1], normal: [0.0, -1.0, 0.0, 1.0], uv_offset: down.1 },
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
