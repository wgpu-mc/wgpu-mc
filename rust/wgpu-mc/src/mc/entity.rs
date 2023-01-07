use std::sync::Arc;

use crate::render::atlas::Atlas;
use crate::texture::{BindableTexture, UV};

use crate::render::entity::EntityVertex;
use crate::render::pipeline::WmPipelines;
use crate::wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::{WgpuState, WmRenderer};
use arc_swap::ArcSwap;
use cgmath::{Matrix4, SquareMatrix, Vector3, Vector4};
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::util::BindableBuffer;
use bytemuck::{Pod, Zeroable};
use wgpu::BufferUsages;

pub type Position = (f32, f32, f32);
pub type EntityType = usize;

pub struct EntityManager {
    pub mob_texture_atlas: RwLock<Atlas>,
    pub player_texture_atlas: RwLock<Atlas>,
    pub entity_types: RwLock<Vec<Arc<Entity>>>,
    pub entity_vertex_buffers: ArcSwap<HashMap<usize, Arc<wgpu::BindGroup>>>,
}

impl EntityManager {
    pub fn new(wgpu_state: &WgpuState, pipelines: &WmPipelines) -> Self {
        Self {
            mob_texture_atlas: RwLock::new(Atlas::new(wgpu_state, pipelines, false)),
            //TODO: support resizing the atlas
            player_texture_atlas: RwLock::new(Atlas::new(wgpu_state, pipelines, false)),
            entity_types: RwLock::new(Vec::new()),
            entity_vertex_buffers: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PartTransform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub pivot_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}

impl PartTransform {
    pub fn describe(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(self.scale_x, self.scale_y, self.scale_z)
            * Matrix4::from_translation(cgmath::Vector3::new(
                self.pivot_x / self.scale_x,
                self.pivot_y / self.scale_y,
                self.pivot_z / self.scale_z,
            ))
            * Matrix4::from_angle_z(cgmath::Deg(self.roll))
            * Matrix4::from_angle_x(cgmath::Deg(self.pitch))
            * Matrix4::from_angle_y(cgmath::Deg(self.yaw))
            * Matrix4::from_translation(cgmath::Vector3::new(
                -self.pivot_x / self.scale_x,
                -self.pivot_y / self.scale_y,
                -self.pivot_z / self.scale_z,
            ))
            * Matrix4::from_translation(cgmath::Vector3::new(
                self.x / self.scale_x,
                self.y / self.scale_y,
                self.z / self.scale_z,
            ))
    }

    pub fn identity() -> Self {
        PartTransform {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pivot_x: 0.0,
            pivot_y: 0.0,
            pivot_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CuboidUV {
    pub north: UV,
    pub east: UV,
    pub south: UV,
    pub west: UV,
    pub up: UV,
    pub down: UV,
}

///Cuboid dimensions are in block units (16 block units per block aka 1 meter)
/// Position offsets are in meters
#[derive(Copy, Clone, Debug)]
pub struct Cuboid {
    //X offset of this cuboid in world units
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub width: f32,
    pub height: f32,
    pub length: f32,

    pub textures: CuboidUV,
}

impl Cuboid {
    pub fn describe(&self, matrix: Matrix4<f32>, part_id: u32) -> [[EntityVertex; 6]; 6] {
        let width = self.width / 16.0;
        let height = self.height / 16.0;
        let length = self.length / 16.0;
        let x = self.x / 16.0;
        let y = self.y / 16.0;
        let z = self.z / 16.0;

        let a = (matrix * Vector4::new(x, y, z, 1.0)).truncate().into();
        let b = (matrix * Vector4::new(x + width, y, z, 1.0))
            .truncate()
            .into();
        let c = (matrix * Vector4::new(x + width, y + height, z, 1.0))
            .truncate()
            .into();
        let d = (matrix * Vector4::new(x, y + height, z, 1.0))
            .truncate()
            .into();
        let e = (matrix * Vector4::new(x, y, z + length, 1.0))
            .truncate()
            .into();
        let f = (matrix * Vector4::new(x + width, y, z + length, 1.0))
            .truncate()
            .into();
        let g = (matrix * Vector4::new(x + width, y + height, z + length, 1.0))
            .truncate()
            .into();
        let h = (matrix * Vector4::new(x, y + height, z + length, 1.0))
            .truncate()
            .into();

        [
            [
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.south[1][0], self.textures.south[1][1]],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.south[1][0], self.textures.south[0][1]],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.south[0][0], self.textures.south[1][1]],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.south[1][0], self.textures.south[0][1]],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.south[0][0], self.textures.south[0][1]],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.south[0][0], self.textures.south[1][1]],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.west[1][0], self.textures.west[0][1]],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.west[0][0], self.textures.west[1][1]],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.west[1][0], self.textures.west[1][1]],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.west[0][0], self.textures.west[0][1]],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.west[0][0], self.textures.west[1][1]],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.west[1][0], self.textures.west[0][1]],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.north[1][0], self.textures.north[0][1]],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.north[0][0], self.textures.north[1][1]],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.north[1][0], self.textures.north[1][1]],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.north[0][0], self.textures.north[0][1]],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.north[0][0], self.textures.north[1][1]],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.north[1][0], self.textures.north[0][1]],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.east[0][0], self.textures.east[1][1]],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.east[1][0], self.textures.east[1][1]],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.east[1][0], self.textures.east[0][1]],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.east[1][0], self.textures.east[0][1]],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.east[0][0], self.textures.east[0][1]],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.east[0][0], self.textures.east[1][1]],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.up[1][0], self.textures.up[0][1]],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.up[0][0], self.textures.up[0][1]],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.up[0][0], self.textures.up[1][1]],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.up[1][0], self.textures.up[1][1]],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.up[1][0], self.textures.up[0][1]],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.up[0][0], self.textures.up[1][1]],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.down[0][0], self.textures.down[1][1]],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.down[0][0], self.textures.down[0][1]],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.down[1][0], self.textures.down[0][1]],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.down[0][0], self.textures.down[1][1]],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.down[1][0], self.textures.down[0][1]],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.down[1][0], self.textures.down[1][1]],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
            ],
        ]
    }
}

///A part of an entity model, defined as a transform, some cuboids, and some child [EntityPart]s, which recursively inherit the transforms
/// of their respective parents
#[derive(Clone, Debug)]
pub struct EntityPart {
    pub name: String,
    pub transform: PartTransform,
    pub cuboids: Vec<Cuboid>,
    pub children: Vec<EntityPart>,
}

///A struct that represents an entity model and it's mesh along with corresponding data
#[derive(Debug)]
pub struct Entity {
    pub name: String,
    pub model_root: EntityPart,
    pub texture: Arc<ArcSwap<BindableTexture>>,
    /// Names of each part referencing an index for applicable transforms
    pub parts: HashMap<String, usize>,
    pub mesh: Arc<wgpu::Buffer>,
    pub vertices: u32,
}

fn recurse_get_mesh(part: &EntityPart, vertices: &mut Vec<EntityVertex>, part_id: &mut u32) {
    part.cuboids.iter().for_each(|cuboid| {
        vertices.extend(
            cuboid
                .describe(Matrix4::identity(), *part_id)
                .iter()
                .copied()
                .flatten(),
        );
    });

    *part_id += 1;

    part.children.iter().for_each(|part| {
        recurse_get_mesh(part, vertices, part_id);
    });
}

fn recurse_get_names(part: &EntityPart, index: &mut usize, names: &mut HashMap<String, usize>) {
    names.insert(part.name.clone(), *index);
    *index += 1;
    part.children
        .iter()
        .for_each(|part| recurse_get_names(part, index, names));
}

impl Entity {
    ///Create an entity from an [EntityPart] and upload it's mesh to the GPU
    pub fn new(
        name: String,
        root: EntityPart,
        wgpu_state: &WgpuState,
        texture: Arc<ArcSwap<BindableTexture>>,
    ) -> Self {
        let mut parts = HashMap::new();

        recurse_get_names(&root, &mut 0, &mut parts);

        let mut mesh = Vec::new();

        let mut part_id = 0;
        recurse_get_mesh(&root, &mut mesh, &mut part_id);

        Self {
            name,
            model_root: root,
            texture,
            parts,
            mesh: Arc::new(wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh[..]),
                usage: wgpu::BufferUsages::VERTEX,
            })),
            vertices: mesh.len() as u32,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct UploadedEntityInstances {
    pub(crate) transform_ssbo: Arc<BindableBuffer>,
    pub(crate) instance_vbo: Arc<wgpu::Buffer>,
    pub(crate) count: u32,
}

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub(crate) struct InstanceVertex {
    /// Index into mat4[]
    pub(crate) entity_index: u32,
    pub(crate) uv_offset: [f32; 2],
    pub(crate) parts_per_entity: u32,
}

impl InstanceVertex {
    const VAA: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        4 => Uint32,
        5 => Float32x2,
        6 => Uint32
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VAA,
        }
    }
}

pub struct EntityInstances {
    pub(crate) entity: Arc<Entity>,
    pub instances: Vec<EntityInstanceTransforms>,
    pub(crate) uploaded: RwLock<Option<UploadedEntityInstances>>,
}

impl EntityInstances {
    pub fn new(entity: Arc<Entity>, instances: Vec<EntityInstanceTransforms>) -> Self {
        Self {
            entity,
            instances,
            uploaded: RwLock::new(None),
        }
    }

    pub fn upload(&self, wm: &WmRenderer) {
        let matrices = self
            .instances
            .iter()
            .flat_map(|transforms| {
                transforms
                    .get_matrices(&self.entity)
                    .into_iter()
                    .flatten()
                    .flatten()
            })
            .collect::<Vec<f32>>();

        let instances: Vec<InstanceVertex> = self
            .instances
            .iter()
            .enumerate()
            .map(|(index, instance)| InstanceVertex {
                entity_index: index as u32,
                uv_offset: [instance.uv_offset.0, instance.uv_offset.1],
                parts_per_entity: self.entity.parts.len() as u32,
            })
            .collect();

        let instances_bytes = bytemuck::cast_slice(&instances[..]);

        let instance_vbo = Arc::new(wm.wgpu_state.device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: instances_bytes,
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        let transform_ssbo = Arc::new(BindableBuffer::new(
            &wm,
            bytemuck::cast_slice(&matrices),
            BufferUsages::STORAGE,
            "ssbo",
        ));

        *self.uploaded.write() = Some(UploadedEntityInstances {
            transform_ssbo,
            instance_vbo,
            count: self.instances.len() as u32,
        });
    }
}

pub struct EntityInstanceTransforms {
    ///Index
    pub position: Position,
    ///Rotation around the Y axis
    pub looking_yaw: f32,
    pub uv_offset: (f32, f32),
    pub part_transforms: Vec<PartTransform>,
}

impl EntityInstanceTransforms {
    pub fn get_matrices(&self, entity: &Entity) -> Vec<[[f32; 4]; 4]> {
        let transforms: Vec<Matrix4<f32>> = self
            .part_transforms
            .iter()
            .map(|pt| pt.describe())
            .collect();

        let mut vec = Vec::new();

        // let mut index = 0;
        recurse_transforms(
            Matrix4::from_translation(Vector3::new(0.5, 0.5, 0.5))
                * Matrix4::from_angle_y(cgmath::Deg(self.looking_yaw))
                * Matrix4::from_translation(Vector3::new(-0.5, -0.5, -0.5))
                * Matrix4::from_translation(cgmath::Vector3::new(
                    self.position.0,
                    self.position.1,
                    self.position.2,
                )),
            &entity.model_root,
            &mut vec,
            // &mut index,
            &transforms[..],
        );

        vec.iter().map(|mat| (*mat).into()).collect()
    }
}

fn recurse_transforms(
    mat: Matrix4<f32>,
    part: &EntityPart,
    vec: &mut Vec<Matrix4<f32>>,
    instance_transforms: &[Matrix4<f32>],
) {
    let instance_part_transform = instance_transforms[0];

    //mat is a transformation matrix that has been composed recursively from it's parent's and ancestors' transforms
    //part.transform.describe() gets the transformation that was described in the model
    //instance_part_transform gets the transform that is being applied to a specific part of a specific instance of an entity

    let new_mat = mat * part.transform.describe() * instance_part_transform;

    vec.push(new_mat);

    let mut slice = &instance_transforms[1..];

    if slice.len() == 0 {
        return;
    }

    part.children.iter().for_each(|child| {
        recurse_transforms(new_mat, child, vec, slice);
        slice = &slice[1..];
    });
}
