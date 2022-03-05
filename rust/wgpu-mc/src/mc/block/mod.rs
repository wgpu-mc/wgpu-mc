



use crate::mc::block::blockstate::{BlockstateVariantDefinitionModel, BlockstateVariantModelDefinitionRotations};
use crate::mc::datapack::{NamespacedResource};




use std::convert::TryFrom;
use indexmap::map::IndexMap;


pub mod model;
pub mod blockstate;

pub struct Block {
    pub id: NamespacedResource,
    pub states: IndexMap<BlockstateVariantKey, BlockstateVariantDefinitionModel>
}

impl Block {

    pub fn from_json(name: &str, json: &str) -> Option<Self> {
        let json_val: serde_json::Value = serde_json::from_str(json).ok()?;
        let states = json_val.as_object()?.get("variants")?.as_object()?.iter().map(|(key, val)| {
            let obj = val.as_object().or_else(|| {
                val.as_array()?.first()?.as_object()
            })?;

            Some((key.clone(), BlockstateVariantDefinitionModel {
                id: NamespacedResource::try_from(key.as_str()).ok()?,
                rotations: BlockstateVariantModelDefinitionRotations {
                    x: obj.get("x").and_then(|num| {
                        Some(num.as_u64()? as u16)
                    }).or(Some(0))?,
                    y: obj.get("y").and_then(|num| {
                        Some(num.as_u64()? as u16)
                    }).or(Some(0))?,
                    z: obj.get("z").and_then(|num| {
                        Some(num.as_u64()? as u16)
                    }).or(Some(0))?
                },
                model: NamespacedResource::try_from(obj.get("model")?.as_str()?).ok()?
            }))
        }).collect::<Option<IndexMap<String, BlockstateVariantDefinitionModel>>>()?;

        Some(Self {
            id: NamespacedResource::try_from(name).ok()?,
            states
        })
    }

}

#[derive(Clone, Copy, Debug, Hash)]
pub enum BlockDirection {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

impl From<&str> for BlockDirection {
    fn from(string: &str) -> Self {
        match &string.to_ascii_lowercase()[..] {
            "north" => Self::North,
            "east" => Self::East,
            "south" => Self::South,
            "west" => Self::West,
            "up" => Self::Up,
            "down" => Self::Down,
            _ => panic!("Invalid block direction")
        }
    }
}

pub type BlockPos = (i32, u8, i32);

pub type BlockstateVariantKey = String;

///First 22 bits (left-to-right) are an index into which `Block` this BlockState represents
/// The last 10 bits are used to describe which variant blockstate this `BlockState` represents
///
/// The entire thing is used as an index into `BlockManager.baked_block_variants`
pub type PackedBlockstateKey = u32;

///The state of one block, describing which variant
#[derive(Clone, Copy, Debug)]
pub struct BlockState {
    pub packed_key: Option<PackedBlockstateKey>
}
