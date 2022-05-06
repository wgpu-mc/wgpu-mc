use std::sync::Arc;

use crate::render::atlas::Atlas;
use crate::texture::UV;

use crate::render::entity::EntityVertex;
use crate::render::pipeline::RenderPipelineManager;
use crate::wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::{WgpuState, WmRenderer};
use arc_swap::ArcSwap;
use cgmath::{Matrix4, SquareMatrix, Vector4};
use parking_lot::RwLock;
use std::collections::HashMap;

pub type Position = (f64, f64, f64);
pub type EntityType = usize;

pub struct EntityManager {
    pub mob_texture_atlas: RwLock<Atlas>,
    pub player_texture_atlas: RwLock<Atlas>,
    pub entity_types: RwLock<Vec<Arc<EntityModel>>>,
    pub entity_vertex_buffers: ArcSwap<HashMap<usize, Arc<wgpu::BindGroup>>>,
}

impl EntityManager {
    pub fn new(wgpu_state: &WgpuState, pipelines: &RenderPipelineManager) -> Self {
        Self {
            mob_texture_atlas: RwLock::new(Atlas::new(wgpu_state, pipelines)),
            player_texture_atlas: RwLock::new(Atlas::new(wgpu_state, pipelines)),
            entity_types: RwLock::new(Vec::new()),
            entity_vertex_buffers: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PartTransform {
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub pivot_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl PartTransform {
    pub fn describe(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            self.pivot_x,
            self.pivot_y,
            self.pivot_z,
        )) * cgmath::Matrix4::from_angle_z(cgmath::Deg(self.roll))
            * cgmath::Matrix4::from_angle_x(cgmath::Deg(self.pitch))
            * cgmath::Matrix4::from_angle_y(cgmath::Deg(self.yaw))
            * cgmath::Matrix4::from_translation(cgmath::Vector3::new(
                -self.pivot_x,
                -self.pivot_y,
                -self.pivot_z,
            ))
    }

    pub fn zero() -> Self {
        PartTransform {
            pivot_x: 0.0,
            pivot_y: 0.0,
            pivot_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
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

#[derive(Copy, Clone, Debug)]
pub struct Cuboid {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub width: f32,
    pub height: f32,
    pub length: f32,

    pub textures: CuboidUV,
}

impl Cuboid {
    pub fn describe(&self, matrix: &Matrix4<f32>, part_id: u32) -> [[EntityVertex; 6]; 6] {
        let a = (matrix * Vector4::new(self.x, self.y, self.z, 1.0))
            .truncate()
            .into();
        let b = (matrix * Vector4::new(self.x + self.width, self.y, self.z, 1.0))
            .truncate()
            .into();
        let c = (matrix * Vector4::new(self.x + self.width, self.y + self.height, self.z, 1.0))
            .truncate()
            .into();
        let d = (matrix * Vector4::new(self.x, self.y + self.height, self.z, 1.0))
            .truncate()
            .into();
        let e = (matrix * Vector4::new(self.x, self.y, self.z + self.length, 1.0))
            .truncate()
            .into();
        let f = (matrix * Vector4::new(self.x + self.width, self.y, self.z + self.length, 1.0))
            .truncate()
            .into();
        let g = (matrix
            * Vector4::new(
                self.x + self.width,
                self.y + self.height,
                self.z + self.length,
                1.0,
            ))
        .truncate()
        .into();
        let h = (matrix * Vector4::new(self.x, self.y + self.height, self.z + self.length, 1.0))
            .truncate()
            .into();

        [
            [
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.south.1 .0, self.textures.south.1 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.south.1 .0, self.textures.south.0 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.south.0 .0, self.textures.south.1 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.south.1 .0, self.textures.south.0 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.south.0 .0, self.textures.south.0 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.south.0 .0, self.textures.south.1 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.west.1 .0, self.textures.west.0 .1],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.west.0 .0, self.textures.west.1 .1],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.west.1 .0, self.textures.west.1 .1],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.west.0 .0, self.textures.west.0 .1],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.west.0 .0, self.textures.west.1 .1],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.west.1 .0, self.textures.west.0 .1],
                    normal: [-1.0, 0.0, 0.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.north.1 .0, self.textures.north.0 .1],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.north.0 .0, self.textures.north.1 .1],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.north.1 .0, self.textures.north.1 .1],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.north.0 .0, self.textures.north.0 .1],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.north.0 .0, self.textures.north.1 .1],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.north.1 .0, self.textures.north.0 .1],
                    normal: [0.0, 0.0, -1.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.east.0 .0, self.textures.east.1 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.east.1 .0, self.textures.east.1 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.east.1 .0, self.textures.east.0 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.east.1 .0, self.textures.east.0 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.east.0 .0, self.textures.east.0 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.east.0 .0, self.textures.east.1 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.up.1 .0, self.textures.up.0 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.up.0 .0, self.textures.up.0 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.up.0 .0, self.textures.up.1 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: c,
                    tex_coords: [self.textures.up.1 .0, self.textures.up.1 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.up.1 .0, self.textures.up.0 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: d,
                    tex_coords: [self.textures.up.0 .0, self.textures.up.1 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
            ],
            [
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.down.1 .0, self.textures.down.0 .1],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.down.0 .0, self.textures.down.0 .1],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.down.0 .0, self.textures.down.1 .1],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.down.1 .0, self.textures.down.1 .1],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: a,
                    tex_coords: [self.textures.down.1 .0, self.textures.down.0 .1],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.down.0 .0, self.textures.down.1 .1],
                    normal: [0.0, -1.0, 0.0],
                    part_id,
                },
            ],
        ]
    }
}

#[derive(Clone, Debug)]
pub struct EntityPart {
    pub name: Arc<String>,
    pub transform: PartTransform,
    pub cuboids: Vec<Cuboid>,
    pub children: Vec<EntityPart>,
}

#[derive(Clone, Debug)]
pub struct EntityModel {
    pub root: EntityPart,
    /// Names of each part referencing an index for applicable transforms
    pub parts: HashMap<String, usize>,
}

fn recurse_get_mesh(part: &EntityPart, vertices: &mut Vec<EntityVertex>, part_id: &mut u32) {
    let mat = part.transform.describe();

    part.cuboids.iter().for_each(|cuboid| {
        vertices.extend(cuboid.describe(&mat, *part_id).iter().copied().flatten());
    });

    *part_id += 1;

    part.children.iter().for_each(|part| {
        recurse_get_mesh(part, vertices, part_id);
    });
}

fn recurse_get_names(part: &EntityPart, index: &mut usize, names: &mut HashMap<String, usize>) {
    names.insert((*part.name).clone(), *index);
    *index += 1;
    part.children
        .iter()
        .for_each(|part| recurse_get_names(part, index, names));
}

impl EntityModel {
    pub fn new(root: EntityPart) -> Self {
        let mut parts = HashMap::new();

        recurse_get_names(&root, &mut 0, &mut parts);

        Self { root, parts }
    }

    pub fn generate_mesh(&self) -> Vec<EntityVertex> {
        let mut out = Vec::new();

        let mut part_id = 0;
        recurse_get_mesh(&self.root, &mut out, &mut part_id);

        out
    }
}

pub struct DescribedEntityInstances {
    ///the mat4[] for part transforms that's found in the shader
    pub matrices: Vec<Vec<[[f32; 4]; 4]>>,
}

pub type UploadedEntityInstanceBuffer = (wgpu::Buffer, wgpu::BindGroup);

impl DescribedEntityInstances {
    pub fn upload(&self, wm: &WmRenderer) -> UploadedEntityInstanceBuffer {
        let cast_matrices = self
            .matrices
            .iter()
            .flat_map(|vec1| vec1.iter().flatten().flatten())
            .copied()
            .collect::<Vec<f32>>();

        let buffer = wm
            .wgpu_state
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&cast_matrices[..]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let pipelines = wm.render_pipeline_manager.load();

        let bind_group = wm
            .wgpu_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: pipelines.bind_group_layouts.read().get("ssbo").unwrap(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        (buffer, bind_group)
    }
}

pub struct EntityInstance {
    ///Index
    pub entity_model: usize,
    pub position: Position,
    ///Rotation around the Y axis (yaw)
    pub looking_yaw: f32,
    pub uv_offset: (f32, f32),
    ///Paint it red. Or something
    pub hurt: bool,
    pub part_transforms: Vec<PartTransform>,
}

impl EntityInstance {
    pub fn describe_instance(&self, entity_manager: &EntityManager) -> Vec<[[f32; 4]; 4]> {
        let model: &Arc<EntityModel> = &entity_manager.entity_types.read()[self.entity_model];

        let transforms: Vec<Matrix4<f32>> = self
            .part_transforms
            .iter()
            .map(|pt| pt.describe())
            .collect();

        let mut vec = Vec::new();

        let mut index = 0;
        recurse_transforms(
            cgmath::Matrix4::identity(),
            &model.root,
            &mut vec,
            &mut index,
            &transforms[..],
        );

        vec.iter().map(|mat| (*mat).into()).collect()
    }
}

fn recurse_transforms(
    mat: cgmath::Matrix4<f32>,
    part: &EntityPart,
    vec: &mut Vec<cgmath::Matrix4<f32>>,
    index: &mut usize,
    instance_transforms: &[cgmath::Matrix4<f32>],
) {
    let instance_part_transform = instance_transforms[*index];
    let new_mat = mat * part.transform.describe() * instance_part_transform;

    vec.push(new_mat);

    part.children.iter().for_each(|child| {
        *index += 1;
        recurse_transforms(new_mat, child, vec, index, instance_transforms);
    });
}
