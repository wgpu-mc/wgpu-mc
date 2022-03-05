use std::sync::Arc;
use crate::mc::datapack::{NamespacedResource};
use crate::texture::UV;
use crate::render::atlas::Atlas;
use indexmap::map::IndexMap;
use std::collections::HashMap;
use cgmath::{Matrix4, SquareMatrix, Vector4};
use crate::render::entity::EntityVertex;
use arc_swap::ArcSwap;

pub type Position = (f64, f64, f64);
pub type EntityType = usize;

pub struct EntityManager {
    pub mob_texture_atlas: Arc<ArcSwap<Atlas>>,
    pub player_texture_atlas: Atlas,
    pub player_type: Arc<EntityModel>,
    pub entity_types: IndexMap<NamespacedResource, Arc<EntityModel>>,
    pub entity_instance_buffers: ArcSwap<HashMap<usize, Arc<wgpu::BindGroup>>>
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
            roll: 0.0
        }
    }

}

#[derive(Copy, Clone, Debug)]
pub struct CuboidTextures {
    pub north: UV,
    pub east: UV,
    pub south: UV,
    pub west: UV,
    pub up: UV,
    pub down: UV
}

#[derive(Copy, Clone, Debug)]
pub struct Cuboid {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub width: f32,
    pub length: f32,
    pub height: f32,

    pub textures: CuboidTextures
}

impl Cuboid {

    pub fn describe(&self, matrix: &Matrix4<f32>, part_id: u32) -> [[EntityVertex; 6]; 6] {
        let a = (matrix * Vector4::new(self.x, self.y, self.z, 1.0)).truncate().into();
        let b = (matrix * Vector4::new(self.x + self.width, self.y, self.z, 1.0)).truncate().into();
        let c = (matrix * Vector4::new(self.x + self.width, self.y + self.height, self.z, 1.0)).truncate().into();
        let d = (matrix * Vector4::new(self.x, self.y + self.height, self.z, 1.0)).truncate().into();
        let e = (matrix * Vector4::new(self.x, self.y, self.z + self.length, 1.0)).truncate().into();
        let f = (matrix * Vector4::new(self.x + self.width, self.y, self.z + self.length, 1.0)).truncate().into();
        let g = (matrix * Vector4::new(self.x + self.width, self.y + self.height, self.z + self.length, 1.0)).truncate().into();
        let h = (matrix * Vector4::new(self.x, self.y + self.height, self.z + self.length, 1.0)).truncate().into();

        [
            [
                EntityVertex { position: e, tex_coords: [self.textures.south.1.0, self.textures.south.1.1], normal: [0.0, 0.0, 1.0], part_id },
                EntityVertex { position: h, tex_coords: [self.textures.south.1.0, self.textures.south.0.1], normal: [0.0, 0.0, 1.0], part_id },
                EntityVertex { position: f, tex_coords: [self.textures.south.0.0, self.textures.south.1.1], normal: [0.0, 0.0, 1.0], part_id },
                EntityVertex { position: h, tex_coords: [self.textures.south.1.0, self.textures.south.0.1], normal: [0.0, 0.0, 1.0], part_id },
                EntityVertex { position: g, tex_coords: [self.textures.south.0.0, self.textures.south.0.1], normal: [0.0, 0.0, 1.0], part_id },
                EntityVertex { position: f, tex_coords: [self.textures.south.0.0, self.textures.south.1.1], normal: [0.0, 0.0, 1.0], part_id },
            ],
            [
                EntityVertex { position: g, tex_coords: [self.textures.west.1.0, self.textures.west.0.1], normal: [-1.0, 0.0, 0.0], part_id },
                EntityVertex { position: b, tex_coords: [self.textures.west.0.0, self.textures.west.1.1], normal: [-1.0, 0.0, 0.0], part_id },
                EntityVertex { position: f, tex_coords: [self.textures.west.1.0, self.textures.west.1.1], normal: [-1.0, 0.0, 0.0], part_id },
                EntityVertex { position: c, tex_coords: [self.textures.west.0.0, self.textures.west.0.1], normal: [-1.0, 0.0, 0.0], part_id },
                EntityVertex { position: b, tex_coords: [self.textures.west.0.0, self.textures.west.1.1], normal: [-1.0, 0.0, 0.0], part_id },
                EntityVertex { position: g, tex_coords: [self.textures.west.1.0, self.textures.west.0.1], normal: [-1.0, 0.0, 0.0], part_id },
            ],
            [
                EntityVertex { position: c, tex_coords: [self.textures.north.1.0, self.textures.north.0.1], normal: [0.0, 0.0, -1.0], part_id },
                EntityVertex { position: a, tex_coords: [self.textures.north.0.0, self.textures.north.1.1], normal: [0.0, 0.0, -1.0], part_id },
                EntityVertex { position: b, tex_coords: [self.textures.north.1.0, self.textures.north.1.1], normal: [0.0, 0.0, -1.0], part_id },
                EntityVertex { position: d, tex_coords: [self.textures.north.0.0, self.textures.north.0.1], normal: [0.0, 0.0, -1.0], part_id },
                EntityVertex { position: a, tex_coords: [self.textures.north.0.0, self.textures.north.1.1], normal: [0.0, 0.0, -1.0], part_id },
                EntityVertex { position: c, tex_coords: [self.textures.north.1.0, self.textures.north.0.1], normal: [0.0, 0.0, -1.0], part_id },
            ],
            [
                EntityVertex { position: e, tex_coords: [self.textures.east.0.0, self.textures.east.1.1], normal: [1.0, 0.0, 0.0], part_id },
                EntityVertex { position: a, tex_coords: [self.textures.east.1.0, self.textures.east.1.1], normal: [1.0, 0.0, 0.0], part_id },
                EntityVertex { position: d, tex_coords: [self.textures.east.1.0, self.textures.east.0.1], normal: [1.0, 0.0, 0.0], part_id },
                EntityVertex { position: d, tex_coords: [self.textures.east.1.0, self.textures.east.0.1], normal: [1.0, 0.0, 0.0], part_id },
                EntityVertex { position: h, tex_coords: [self.textures.east.0.0, self.textures.east.0.1], normal: [1.0, 0.0, 0.0], part_id },
                EntityVertex { position: e, tex_coords: [self.textures.east.0.0, self.textures.east.1.1], normal: [1.0, 0.0, 0.0], part_id },
            ],
            [
                EntityVertex { position: g, tex_coords: [self.textures.up.1.0, self.textures.up.0.1], normal: [0.0, 1.0, 0.0], part_id },
                EntityVertex { position: h, tex_coords: [self.textures.up.0.0, self.textures.up.0.1], normal: [0.0, 1.0, 0.0], part_id },
                EntityVertex { position: d, tex_coords: [self.textures.up.0.0, self.textures.up.1.1], normal: [0.0, 1.0, 0.0], part_id },
                EntityVertex { position: c, tex_coords: [self.textures.up.1.0, self.textures.up.1.1], normal: [0.0, 1.0, 0.0], part_id },
                EntityVertex { position: g, tex_coords: [self.textures.up.1.0, self.textures.up.0.1], normal: [0.0, 1.0, 0.0], part_id },
                EntityVertex { position: d, tex_coords: [self.textures.up.0.0, self.textures.up.1.1], normal: [0.0, 1.0, 0.0], part_id },
            ],
            [
                EntityVertex { position: a, tex_coords: [self.textures.down.1.0, self.textures.down.0.1], normal: [0.0, -1.0, 0.0], part_id },
                EntityVertex { position: b, tex_coords: [self.textures.down.0.0, self.textures.down.0.1], normal: [0.0, -1.0, 0.0], part_id },
                EntityVertex { position: f, tex_coords: [self.textures.down.0.0, self.textures.down.1.1], normal: [0.0, -1.0, 0.0], part_id },
                EntityVertex { position: e, tex_coords: [self.textures.down.1.0, self.textures.down.1.1], normal: [0.0, -1.0, 0.0], part_id },
                EntityVertex { position: a, tex_coords: [self.textures.down.1.0, self.textures.down.0.1], normal: [0.0, -1.0, 0.0], part_id },
                EntityVertex { position: f, tex_coords: [self.textures.down.0.0, self.textures.down.1.1], normal: [0.0, -1.0, 0.0], part_id },
            ]
        ]
    }

}

#[derive(Clone, Debug)]
pub struct EntityPart {
    pub transform: PartTransform,
    pub cuboids: Vec<Cuboid>,
    pub children: Vec<EntityPart>
}

#[derive(Clone, Debug)]
pub struct EntityModel {
    root: EntityPart,
    /// Names of each part referencing an index for applicable transforms
    parts: HashMap<String, usize>
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

impl EntityModel {

    pub fn get_mesh(&self) -> Vec<EntityVertex> {
        let mut out = Vec::new();

        let mut part_id = 0;
        recurse_get_mesh(&self.root, &mut out, &mut part_id);

        out
    }

}

pub struct EntityInstance {
    entity_model: usize,
    position: Position,
    ///Rotation around the Y axis (yaw)
    looking_yaw: f32,
    uv_offset: (f32, f32),
    hurt: bool,
    part_transforms: Vec<PartTransform>,
}

fn recurse_transforms(
    mat: cgmath::Matrix4<f32>,
    part: &EntityPart,
    vec: &mut Vec<cgmath::Matrix4<f32>>,
    index: &mut usize,
    instance_transforms: &[cgmath::Matrix4<f32>]) {
    let instance_part_transform = instance_transforms[*index];
    let new_mat = mat * part.transform.describe() * instance_part_transform;

    vec.push(
        new_mat
    );

    part.children.iter().for_each(|child| {
        *index += 1;
        recurse_transforms(new_mat, child, vec, index, instance_transforms);
    });
}

impl EntityInstance {

    pub fn describe_instance(&self, entity_manager: &EntityManager) -> Vec<[[f32; 4]; 4]> {
        let (_entity_name, model): (&NamespacedResource, &Arc<EntityModel>) =
            entity_manager.entity_types.get_index(self.entity_model).unwrap();

        let transforms: Vec<Matrix4<f32>> =
            self.part_transforms.iter().map(|pt| pt.describe())
                .collect();

        let mut vec = Vec::new();

        let mut index = 0;
        recurse_transforms(
            cgmath::Matrix4::identity(),
            &model.root,
            &mut vec,
            &mut index,
            &transforms[..]
        );

        vec.iter().map(|mat| (*mat).into()).collect()
    }

}