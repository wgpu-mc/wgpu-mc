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

pub fn tmd_to_wm(part: &ModelPartData) -> Option<EntityPart> {
    Some(EntityPart {
        name: Arc::new("".into()),
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
                Some(Cuboid {
                    x: *cuboid_data.offset.get("x")?,
                    y: *cuboid_data.offset.get("y")?,
                    z: *cuboid_data.offset.get("z")?,
                    width: *cuboid_data.dimensions.get("x")?,
                    height: *cuboid_data.dimensions.get("y")?,
                    length: *cuboid_data.dimensions.get("z")?,
                    textures: CuboidUV {
                        //TODO
                        north: ((0.0, 0.0), (0.0, 0.0)),
                        east: ((0.0, 0.0), (0.0, 0.0)),
                        south: ((0.0, 0.0), (0.0, 0.0)),
                        west: ((0.0, 0.0), (0.0, 0.0)),
                        up: ((0.0, 0.0), (0.0, 0.0)),
                        down: ((0.0, 0.0), (0.0, 0.0)),
                    },
                })
            })
            .collect::<Option<Vec<Cuboid>>>()?,
        children: part
            .children
            .values()
            .map(tmd_to_wm)
            .collect::<Option<Vec<EntityPart>>>()?,
    })
}
