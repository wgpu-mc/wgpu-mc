use std::{collections::HashMap, sync::Arc};

use once_cell::sync::OnceCell;
use serde::Deserialize;

use wgpu_mc::{
    mc::entity::{Cuboid, CuboidUV, EntityPart, PartTransform},
    render::atlas::Atlas,
};

pub static ENTITY_ATLAS: OnceCell<Arc<Atlas>> = OnceCell::new();

#[derive(Debug, Deserialize)]
pub struct ModelCuboidData {
    pub name: Option<String>,
    pub offset: HashMap<String, f32>,
    pub dimensions: HashMap<String, f32>,
    pub mirror: bool,
    #[serde(rename(deserialize = "textureUV"))]
    pub texture_uv: HashMap<String, f32>,
    #[serde(rename(deserialize = "textureScale"))]
    pub texture_scale: HashMap<String, f32>,
}

#[derive(Debug, Deserialize)]
pub struct ModelTransform {
    #[serde(rename(deserialize = "pivotX"))]
    pub pivot_x: f32,
    #[serde(rename(deserialize = "pivotY"))]
    pub pivot_y: f32,
    #[serde(rename(deserialize = "pivotZ"))]
    pub pivot_z: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Debug, Deserialize)]
pub struct ModelPartData {
    #[serde(rename(deserialize = "cuboidData"))]
    pub cuboid_data: Vec<ModelCuboidData>,
    #[serde(rename(deserialize = "rotationData"))]
    pub transform: ModelTransform,
    pub children: HashMap<String, ModelPartData>,
}

#[derive(Debug, Deserialize)]
pub struct ModelData {
    pub data: ModelPartData,
}

#[derive(Debug, Deserialize)]
pub struct TextureDimensions {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
pub struct TexturedModelData {
    pub data: ModelData,
    pub dimensions: TextureDimensions,
}

#[derive(Debug, Copy, Clone)]
pub struct AtlasPosition {
    pub width: u32,
    pub height: u32,
    pub x: f32,
    pub y: f32
}

impl AtlasPosition {

    pub fn map(&self, pos: [f32; 2]) -> [f32; 2] {
        [
            (self.x + pos[0]) / (self.width as f32),
            (self.y + pos[1]) / (self.height as f32),
        ]
    }

}

pub fn tmd_to_wm(name: String, part: &ModelPartData, ap: &AtlasPosition) -> Option<EntityPart> {
    Some(EntityPart {
        name,
        transform: PartTransform {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pivot_x: part.transform.pivot_x,
            pivot_y: part.transform.pivot_y,
            pivot_z: part.transform.pivot_z,
            yaw: part.transform.yaw,
            pitch: part.transform.pitch,
            roll: part.transform.roll,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
        },
        cuboids: part
            .cuboid_data
            .iter()
            .map(|cuboid_data| {
                let pos = [*cuboid_data.texture_uv.get("x").unwrap(), *cuboid_data.texture_uv.get("y").unwrap()];
                let dimensions = [
                    cuboid_data.dimensions.get("x").unwrap(),
                    cuboid_data.dimensions.get("y").unwrap(),
                    cuboid_data.dimensions.get("z").unwrap(),
                ];

                Some(Cuboid {
                    x: *cuboid_data.offset.get("x")?,
                    y: *cuboid_data.offset.get("y")?,
                    z: *cuboid_data.offset.get("z")?,
                    width: *cuboid_data.dimensions.get("x")?,
                    height: *cuboid_data.dimensions.get("y")?,
                    length: *cuboid_data.dimensions.get("z")?,
                    textures: CuboidUV {
                        west: [
                            ap.map([pos[0], pos[1] + dimensions[2]]),
                            ap.map([pos[0] + dimensions[0], pos[1] + (dimensions[2] + dimensions[1])]),
                        ],
                        east: [
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1] + dimensions[2]]),
                            ap.map([(pos[0] + (dimensions[0] * 3.0)), pos[1] + (dimensions[2] + dimensions[1])]),
                        ],
                        south: [
                            ap.map([(pos[0] + (dimensions[0])), pos[1] + dimensions[2]]),
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1] + (dimensions[2] + dimensions[1])]),
                        ],
                        north: [
                            ap.map([(pos[0] + (dimensions[0] * 3.0)), pos[1] + dimensions[2]]),
                            ap.map([(pos[0] + (dimensions[0] * 4.0)), pos[1] + (dimensions[2] + dimensions[1])]),
                        ],
                        up: [
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1]]),
                            ap.map([(pos[0] + (dimensions[0] * 3.0)), pos[1] + (dimensions[2])]),
                        ],
                        down: [
                            ap.map([(pos[0] + dimensions[0]), pos[1]]),
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1] + (dimensions[2])]),
                        ]
                    },
                })
            })
            .collect::<Option<Vec<Cuboid>>>()?,
        children: part
            .children
            .iter()
            .map(|(name, part)| tmd_to_wm(name.clone(), part, ap))
            .collect::<Option<Vec<EntityPart>>>()?,
    })
}
