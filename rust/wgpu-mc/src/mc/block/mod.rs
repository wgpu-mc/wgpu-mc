use std::collections::HashMap;
use crate::mc::block::blockstate::{
    BlockstateVariantDefinitionModel, BlockstateVariantModelDefinitionRotations,
};
use crate::mc::datapack::{NamespacedResource, TextureVariableOrResource};

use indexmap::map::IndexMap;
use std::convert::TryFrom;
use serde_json::Value;

pub mod blockstate;
pub mod model;

pub struct MultipartPredicate {
    pub key: String,
    pub value: String
}

pub struct MultipartApply {
    pub model: TextureVariableOrResource,
    pub x: u8,
    pub z: u8,
    pub uvlock: bool,
    pub weight: u16
}

pub struct MultipartCase {
    pub predicates: Vec<MultipartPredicate>,
    pub apply: Vec<MultipartApply>
}

pub struct Multipart {
    pub cases: Vec<MultipartCase>
}

impl Multipart {

    pub fn generate(&self, keys: &HashMap<String, String>) -> &[&MultipartApply] {
        &self.cases.iter().filter_map(|case| {
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

pub enum BlockDefinition {
    Multipart {
         multipart: Multipart
    },
    Variants {
        states: IndexMap<BlockstateVariantKey, BlockstateVariantDefinitionModel>
    }
}

pub struct Block {
    pub id: NamespacedResource,
    pub definition: BlockDefinition
}

impl Block {
    fn parse_multipart(name: &str, json: &serde_json::Value) -> Option<Self> {
        Some(Self {
            id: name.try_into().ok()?,
            definition: BlockDefinition::Multipart {
                multipart: Multipart {
                    cases: json.as_array()?
                        .iter()
                        .map(|case| {
                            let case = case.as_object()?;

                            let mut multipart_case = MultipartCase {
                                predicates: Vec::new(),
                                apply: Vec::new()
                            };

                            match case.get("when") {
                                None => {}
                                Some(when) => {
                                    let when = when.as_object()?;

                                    if when.contains_key("or") {
                                        multipart_case.predicates = when.get("or")?
                                            .as_array()?
                                            .iter()
                                            .map(|when_entry| {
                                                let when = when_entry.as_object()?
                                                    .iter()
                                                    .next()?;

                                                Some(
                                                    MultipartPredicate {
                                                        key: when.0.clone(),
                                                        value: when.1.as_str().unwrap().into()
                                                    }
                                                )
                                            })
                                            .collect::<Option<Vec<MultipartPredicate>>>()?;
                                    } else {
                                        let when_entry = when
                                            .iter()
                                            .next()?;

                                        multipart_case.predicates.push(
                                            MultipartPredicate {
                                                key: when_entry.0.clone(),
                                                value: when_entry.1.as_str().unwrap().into()
                                            }
                                        );
                                    }
                                }
                            };

                            let apply = case.get("apply")?;

                            match apply {
                                Value::Array(applies) => {
                                    multipart_case.apply = applies
                                        .iter()
                                        .map(|apply| {
                                            let apply = apply.as_object()?;

                                            Some(
                                                MultipartApply {
                                                    model: apply.get("model")?.as_str()?
                                                        .try_into().ok()?,
                                                    x: match apply.get("x") {
                                                        None => 0,
                                                        Some(val) => val.as_u64()? as u8
                                                    },
                                                    z: match apply.get("z") {
                                                        None => 0,
                                                        Some(val) => val.as_u64()? as u8
                                                    },
                                                    uvlock: match apply.get("uvlock") {
                                                        None => false,
                                                        Some(val) => val.as_bool()?
                                                    },
                                                    weight: match apply.get("weight") {
                                                        None => 1,
                                                        Some(val) => val.as_u64()? as u16
                                                    },
                                                }
                                            )
                                        })
                                        .collect::<Option<Vec<MultipartApply>>>()?;
                                }
                                Value::Object(apply) => {
                                    multipart_case.apply.push(
                                        MultipartApply {
                                            model: apply.get("model")?.as_str()?
                                                .try_into().ok()?,
                                            x: match apply.get("x") {
                                                None => 0,
                                                Some(val) => val.as_u64()? as u8
                                            },
                                            z: match apply.get("z") {
                                                None => 0,
                                                Some(val) => val.as_u64()? as u8
                                            },
                                            uvlock: match apply.get("uvlock") {
                                                None => false,
                                                Some(val) => val.as_bool()?
                                            },
                                            weight: match apply.get("weight") {
                                                None => 1,
                                                Some(val) => val.as_u64()? as u16
                                            },
                                        }
                                    );
                                },
                                _ => None?
                            }

                            Some(
                                multipart_case
                            )
                        })
                        .collect::<Option<Vec<MultipartCase>>>()?
                }
            }
        })
    }

    fn parse_variants(name: &str, json: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        let variants = json.iter()
            .map(|(key, val)| {
                let obj = val
                    .as_object()
                    .or_else(|| val.as_array()?.first()?.as_object())?;

                Some((
                    key.clone(),
                    BlockstateVariantDefinitionModel {
                        id: NamespacedResource::try_from(key.as_str()).ok()?,
                        rotations: BlockstateVariantModelDefinitionRotations {
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
            .collect::<Option<IndexMap<String, BlockstateVariantDefinitionModel>>>()?;

        Some(Self {
            id: name.try_into().ok()?,
            definition: BlockDefinition::Variants {
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
            Self::parse_multipart(name, object.get("multipart")?)
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

pub type BlockstateVariantKey = String;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BlockstateKey {
    block_key: u16,
    state_index: u16
}

impl BlockstateKey {

    pub fn new(block_key: u16, state_index: u16) -> Self {
        Self {
            block_key,
            state_index
        }
    }

    pub fn pack(&self) -> u32 {
        ((self.block_key as u32) << 16) | (self.state_index as u32)
    }

    pub fn state_index(&self) -> u16 {
        self.state_index
    }

    pub fn block_key(&self) -> u16 {
        self.block_key
    }

}

impl Into<u32> for BlockstateKey {
    fn into(self) -> u32 {
        self.pack()
    }
}

impl From<u32> for BlockstateKey {

    fn from(num: u32) -> Self {
        Self::new((num >> 16) as u16, (num & 0xffff) as u16)
    }

}

///The state of one block, describing which variant
#[derive(Clone, Copy, Debug)]
pub struct BlockState {
    pub packed_key: Option<BlockstateKey>,
}
