use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Arc;

use arc_swap::ArcSwap;
use bytemuck::{Pod, Zeroable};
use glam::{vec3, vec4, Mat4};
use parking_lot::RwLock;
use wgpu::{BufferDescriptor, BufferUsages};

use crate::render::atlas::Atlas;
use crate::render::entity::EntityVertex;
use crate::texture::UV;
use crate::{Display, WmRenderer};

pub type Position = (f32, f32, f32);
pub type EntityType = usize;

pub struct EntityManager {
    pub mob_texture_atlas: RwLock<Atlas>,
    pub player_texture_atlas: RwLock<Atlas>,
    pub entity_types: RwLock<Vec<Arc<Entity>>>,
    pub entity_vertex_buffers: ArcSwap<HashMap<usize, Arc<wgpu::BindGroup>>>,
}

impl EntityManager {
    pub fn new(wgpu_state: &Display) -> Self {
        Self {
            mob_texture_atlas: RwLock::new(Atlas::new(wgpu_state, false)),
            //TODO: support resizing the atlas
            player_texture_atlas: RwLock::new(Atlas::new(wgpu_state, false)),
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
const DEG_TO_RAD: f32 = PI / 180.0;
impl PartTransform {
    pub fn describe(&self) -> Mat4 {
        Mat4::from_scale(vec3(self.scale_x, self.scale_y, self.scale_z))
            * Mat4::from_translation(vec3(
                self.pivot_x / self.scale_x,
                self.pivot_y / self.scale_y,
                self.pivot_z / self.scale_z,
            ))
            * Mat4::from_rotation_z(self.roll * DEG_TO_RAD)
            * Mat4::from_rotation_x(self.pitch * DEG_TO_RAD)
            * Mat4::from_rotation_y(self.yaw * DEG_TO_RAD)
            * Mat4::from_translation(vec3(
                -self.pivot_x / self.scale_x,
                -self.pivot_y / self.scale_y,
                -self.pivot_z / self.scale_z,
            ))
            * Mat4::from_translation(vec3(
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
    pub fn describe(&self, matrix: Mat4, part_id: u32) -> [[EntityVertex; 6]; 6] {
        let width = self.width / 16.0;
        let height = self.height / 16.0;
        let length = self.length / 16.0;
        let x = self.x / 16.0;
        let y = self.y / 16.0;
        let z = self.z / 16.0;

        let a = (matrix * vec4(x, y, z, 1.0)).truncate().into();
        let b = (matrix * vec4(x + width, y, z, 1.0)).truncate().into();
        let c = (matrix * vec4(x + width, y + height, z, 1.0))
            .truncate()
            .into();
        let d = (matrix * vec4(x, y + height, z, 1.0)).truncate().into();
        let e = (matrix * vec4(x, y, z + length, 1.0)).truncate().into();
        let f = (matrix * vec4(x + width, y, z + length, 1.0))
            .truncate()
            .into();
        let g = (matrix * vec4(x + width, y + height, z + length, 1.0))
            .truncate()
            .into();
        let h = (matrix * vec4(x, y + height, z + length, 1.0))
            .truncate()
            .into();

        [
            [
                EntityVertex {
                    position: h,
                    tex_coords: [self.textures.south.1 .0, self.textures.south.0 .1],
                    normal: [0.0, 0.0, 1.0],
                    part_id,
                },
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.south.1 .0, self.textures.south.1 .1],
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
                    position: g,
                    tex_coords: [self.textures.south.0 .0, self.textures.south.0 .1],
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
            ],
            [
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
                EntityVertex {
                    position: f,
                    tex_coords: [self.textures.west.1 .0, self.textures.west.1 .1],
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
                    position: c,
                    tex_coords: [self.textures.west.0 .0, self.textures.west.0 .1],
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
                EntityVertex {
                    position: b,
                    tex_coords: [self.textures.north.1 .0, self.textures.north.1 .1],
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
                    position: d,
                    tex_coords: [self.textures.north.0 .0, self.textures.north.0 .1],
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
                    position: a,
                    tex_coords: [self.textures.east.1 .0, self.textures.east.1 .1],
                    normal: [1.0, 0.0, 0.0],
                    part_id,
                },
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.east.0 .0, self.textures.east.1 .1],
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
                    position: d,
                    tex_coords: [self.textures.east.1 .0, self.textures.east.0 .1],
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
                    position: h,
                    tex_coords: [self.textures.up.0 .0, self.textures.up.0 .1],
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
                EntityVertex {
                    position: g,
                    tex_coords: [self.textures.up.1 .0, self.textures.up.0 .1],
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
                    position: d,
                    tex_coords: [self.textures.up.0 .0, self.textures.up.1 .1],
                    normal: [0.0, 1.0, 0.0],
                    part_id,
                },
            ],
            [
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
                    position: a,
                    tex_coords: [self.textures.down.1 .0, self.textures.down.0 .1],
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
                EntityVertex {
                    position: e,
                    tex_coords: [self.textures.down.1 .0, self.textures.down.1 .1],
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
    /// Names of each part referencing an index for applicable transforms
    pub parts: HashMap<String, usize>,
    pub mesh: Arc<wgpu::Buffer>,
    pub vertex_count: u32,
}

fn recurse_get_mesh(part: &EntityPart, vertices: &mut Vec<EntityVertex>, part_id: &mut u32) {
    part.cuboids.iter().for_each(|cuboid| {
        vertices.extend(
            cuboid
                .describe(Mat4::IDENTITY, *part_id)
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
    pub fn new(name: String, root: EntityPart, wgpu_state: &Display) -> Self {
        let mut parts = HashMap::new();

        recurse_get_names(&root, &mut 0, &mut parts);

        let mut mesh = Vec::new();

        let mut part_id = 0;
        recurse_get_mesh(&root, &mut mesh, &mut part_id);
        let buffer = wgpu_state.device.create_buffer(&BufferDescriptor {
            //create buffer init get stuck idk why
            label: None,
            size: (mesh.len() * std::mem::size_of::<EntityVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        wgpu_state
            .queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(&mesh));
        Self {
            name,
            model_root: root,
            parts,
            mesh: Arc::new(buffer),
            vertex_count: mesh.len() as u32,
        }
    }
}

#[derive(Clone)]
pub struct UploadedEntityInstances {
    pub bind_group: Arc<wgpu::BindGroup>,
    pub transforms_buffer: Arc<wgpu::Buffer>,
    pub instance_vbo: Arc<wgpu::Buffer>,
    pub len: u32,
}

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct InstanceVertex {
    pub uv_offset: [u16; 2],
    pub overlay: u32,
}

impl InstanceVertex {
    const VAA: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        4 => Float32x2,
        5 => Uint32
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

#[derive(Clone)]
pub struct BundledEntityInstances {
    pub entity: Arc<Entity>,
    pub uploaded: UploadedEntityInstances,
    pub capacity: u32,
}

impl BundledEntityInstances {
    pub fn new(
        wm: &WmRenderer,
        entity: Arc<Entity>,
        texture_view: &wgpu::TextureView,
        capacity: u32,
    ) -> Self {
        let transforms_buffer = Arc::new(wm.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: capacity as wgpu::BufferAddress
                * (entity.parts.len() as wgpu::BufferAddress)
                * 64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        Self {
            entity,
            uploaded: UploadedEntityInstances {
                bind_group: Arc::new(wm.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: wm.bind_group_layouts.get("entity").unwrap(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: transforms_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(texture_view),
                        },
                    ],
                })),
                transforms_buffer,
                instance_vbo: Arc::new(wm.gpu.device.create_buffer(&BufferDescriptor {
                    label: None,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    size: 100000,
                    mapped_at_creation: false,
                })),
                len: capacity,
            },
            capacity,
        }
    }

    // pub fn upload(&mut self, wm: &WmRenderer, instances: &[EntityInstance]) {
    //     self.count = instances.len() as u32;
    //
    //     let matrices = instances
    //         .iter()
    //         .flat_map(|transforms| {
    //             transforms
    //                 .get_matrices(&self.entity)
    //                 .into_iter()
    //                 .flatten()
    //                 .flatten()
    //         })
    //         .collect::<Vec<f32>>();
    //
    //     let instances: Vec<InstanceVertex> = instances
    //         .iter()
    //         .map(|instance| InstanceVertex {
    //             uv_offset: instance.uv_offset,
    //             overlay: instance.overlay,
    //         })
    //         .collect();
    //
    //     let instances_bytes = bytemuck::cast_slice(&instances[..]);
    //
    //     let instance_vbo = Arc::new(wm.display.device.create_buffer_init(&BufferInitDescriptor {
    //         label: None,
    //         contents: instances_bytes,
    //         usage: BufferUsages::VERTEX,
    //     }));
    //
    //     self.uploaded = UploadedEntityInstances {
    //         bind_group: Arc::new(()),
    //         transforms_buffer: Arc::new(()),
    //         instance_vbo,
    //         count: self.count,
    //     };
    // }
}

pub struct EntityInstance {
    ///Index
    pub position: Position,
    ///Rotation around the Y axis
    pub looking_yaw: f32,
    pub uv_offset: [u16; 2],
    pub part_transforms: Vec<PartTransform>,
    pub overlay: u32,
}

impl EntityInstance {
    pub fn get_matrices(&self, entity: &Entity) -> Vec<[[f32; 4]; 4]> {
        let transforms: Vec<Mat4> = self
            .part_transforms
            .iter()
            .map(|pt| pt.describe())
            .collect();

        let mut vec = Vec::new();

        // let mut index = 0;
        recurse_transforms(
            Mat4::from_translation(vec3(0.5, 0.5, 0.5))
                * Mat4::from_rotation_y(self.looking_yaw * DEG_TO_RAD)
                * Mat4::from_translation(vec3(-0.5, -0.5, -0.5))
                * Mat4::from_translation(vec3(self.position.0, self.position.1, self.position.2)),
            &entity.model_root,
            &mut vec,
            // &mut index,
            &transforms[..],
        );

        vec.iter().map(|mat| mat.to_cols_array_2d()).collect()
    }
}

fn recurse_transforms(
    mat: Mat4,
    part: &EntityPart,
    vec: &mut Vec<Mat4>,
    instance_transforms: &[Mat4],
) {
    let instance_part_transform = instance_transforms[0];

    //mat is a transformation matrix that has been composed recursively from it's parent's and ancestors' transforms
    //part.transform.describe() gets the transformation that was described in the model
    //instance_part_transform gets the transform that is being applied to a specific part of a specific instance of an entity

    let new_mat = mat * part.transform.describe() * instance_part_transform;

    vec.push(new_mat);

    let mut slice = &instance_transforms[1..];

    if slice.is_empty() {
        return;
    }

    part.children.iter().for_each(|child| {
        recurse_transforms(new_mat, child, vec, slice);
        slice = &slice[1..];
    });
}
