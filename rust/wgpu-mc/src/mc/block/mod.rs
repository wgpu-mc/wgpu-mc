use std::collections::HashMap;
use crate::mc::block::blockstate::{
    BlockstateVariantModelDefinition, BlockModelRotations,
};
use crate::mc::datapack::{NamespacedResource, TextureVariableOrResource};

use indexmap::map::IndexMap;
use std::convert::TryFrom;
use std::sync::Arc;
use serde_json::Value;
use crate::mc::block::model::BlockModelMesh;
use crate::mc::block::multipart_json::MultipartJson;
use crate::mc::BlockManager;
use crate::WmRenderer;

pub mod blockstate;
pub mod model;

///Multipart definitions from a JSON datapack
pub mod multipart_json {
    use std::collections::HashMap;
    use serde::{Deserialize};

    #[derive(Debug, Deserialize)]
    pub struct MultipartApplyJson {
        pub model: String,
        #[serde(default)]
        pub x: u8,
        #[serde(default)]
        pub y: u8,
        #[serde(default)]
        pub uvlock: bool,
        #[serde(default = "default_weight")]
        pub weight: u16
    }

    fn default_weight() -> u16 { 1 }

    use serde_with::{serde_as, OneOrMany};
    use serde_with::formats::PreferMany;

    #[serde_as]
    #[derive(Debug, Deserialize)]
    pub struct MultipartCaseJson {
        #[serde_as(deserialize_as = "OneOrMany<_, PreferMany>")]
        pub when: Vec<HashMap<String, String>>,
        #[serde_as(deserialize_as = "OneOrMany<_, PreferMany>")]
        pub apply: Vec<MultipartApplyJson>
    }

    #[derive(Debug)]
    pub struct MultipartJson {
        pub cases: Vec<MultipartCaseJson>
    }

}

#[derive(Debug)]
pub struct MultipartPredicate {
    pub key: String,
    pub value: String
}

#[derive(Debug)]
pub struct MultipartApply {
    pub model: Arc<BlockModelMesh>,
    pub x: u8,
    pub y: u8,
    pub uvlock: bool,
    pub weight: u16
}

#[derive(Debug)]
pub struct MultipartCase {
    pub predicates: Vec<MultipartPredicate>,
    pub apply: Vec<MultipartApply>
}

#[derive(Debug)]
pub struct Multipart {
    pub cases: Vec<MultipartCase>
}

impl Multipart {

    pub fn from_json(json: &MultipartJson, block_manager: &BlockManager) -> Self {
        Self {
            cases: json.cases.iter().map(|case| {
                MultipartCase {
                    predicates: case.when.iter().map(|when| {
                        when.iter().map(|(key, value)| {
                            MultipartPredicate {
                                key: key.clone(),
                                value: value.clone()
                            }
                        })
                    }).flatten().collect(),
                    apply: case.apply.iter().map(|apply| {
                        MultipartApply {
                            model: block_manager.model_meshes.get(
                                &NamespacedResource::try_from(&apply.model[..]).unwrap()
                            ).expect(&apply.model).clone(),
                            x: apply.x,
                            y: apply.y,
                            uvlock: apply.uvlock,
                            weight: apply.weight
                        }
                    }).collect()
                }
            }).collect()
        }

    }

    pub fn generate(&self, keys: &HashMap<String, String>) -> Vec<&MultipartApply> {
        self.cases.iter().filter_map(|case| {
            if case.predicates.iter().any(|predicate|
                keys.get(&predicate.key)
                    .map_or(false, |value| value == &predicate.value)
            ) {
                Some(&case.apply)
            } else {
                None
            }
        }).flatten().collect::<Vec<&MultipartApply>>()
    }

}

#[derive(Debug)]
pub enum BlockDefinitionType {
    Multipart {
         multipart: multipart_json::MultipartJson
    },
    Variants {
        states: HashMap<String, BlockstateVariantModelDefinition>
    }
}

///The representation that a [Block] will be derived from. It's a direct representation of how blockstates are defined in datapacks
#[derive(Debug)]
pub struct BlockDefinition {
    pub id: NamespacedResource,
    pub definition: BlockDefinitionType
}

impl BlockDefinition {

    fn parse_multipart(name: &str, json: serde_json::Value) -> Option<Self> {
        Some(
            Self {
                id: name.try_into().ok()?,
                definition: BlockDefinitionType::Multipart {
                    multipart: MultipartJson {
                        cases: serde_json::from_value(json).ok()?
                    }
                }
            }
        )
    }

    fn parse_variants(name: &str, json: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        let variants = json.iter()
            .map(|(key, val)| {
                let obj = val
                    .as_object()
                    .or_else(|| val.as_array()?.first()?.as_object())?;

                Some((
                    key.clone(),
                    BlockstateVariantModelDefinition {
                        id: NamespacedResource::try_from(key.as_str()).ok()?,
                        rotations: BlockModelRotations {
                            x: obj
                                .get("x")
                                .and_then(|num| Some(num.as_u64()? as u16))
                                .or(Some(0))?,
                            y: obj
                                .get("y")
                                .and_then(|num| Some(num.as_u64()? as u16))
                                .or(Some(0))?,
                            z: obj
                                .get("z")
                                .and_then(|num| Some(num.as_u64()? as u16))
                                .or(Some(0))?,
                        },
                        model: NamespacedResource::try_from(obj.get("model")?.as_str()?).ok()?,
                    },
                ))
            })
            .collect::<Option<HashMap<String, BlockstateVariantModelDefinition>>>()?;

        Some(Self {
            id: name.try_into().ok()?,
            definition: BlockDefinitionType::Variants {
                states: variants.into()
            }
        })
    }

    pub fn from_json(name: &str, json: &str) -> Option<Self> {
        let json_val: serde_json::Value = serde_json::from_str(json).ok()?;
        let object = json_val
            .as_object()?;

        if object.contains_key("variants") {
            Self::parse_variants(name, object.get("variants")?.as_object()?)
        } else {
            Self::parse_multipart(name, object.get("multipart")?.to_owned())
        }
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
            _ => panic!("Invalid block direction"),
        }
    }
}

pub type BlockPos = (i32, u16, i32);

pub type BlockstateKey = u32;

///The state of one block, describing which variant
#[derive(Clone, Copy, Debug)]
pub struct ChunkBlockState {
    pub packed_key: Option<BlockstateKey>,
}
