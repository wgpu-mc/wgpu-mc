use serde_json::{Value, Number};
use std::rc::Rc;
use cgmath::Matrix4;
use std::collections::HashMap;
use serde_bytes::deserialize;
use std::path::PathBuf;
use futures::FutureExt;

pub type NamespacedResource = (String, String);

#[derive(Debug, Clone, Eq, Hash)]
pub enum NamespacedId {
    Tag(String),
    Resource(NamespacedResource),
    Invalid
}

impl NamespacedId {

    pub(crate) fn is_tag(&self) -> bool {
        match self {
            NamespacedId::Tag(_) => true,
            _ => false
        }
    }

}

impl PartialEq for NamespacedId {
    fn eq(&self, other: &Self) -> bool {
        match self {
            NamespacedId::Tag(tag) => match other {
                NamespacedId::Tag(o) => {
                    o == tag
                },
                _ => false,
            },
            NamespacedId::Resource((ns, id)) => {
                match other {
                    NamespacedId::Resource((ons, oid)) => {
                        ons == ns && oid == id
                    }
                    _ => false
                }
            }
            NamespacedId::Invalid => {
                false
            }
        }
    }
}

impl From<&str> for NamespacedId {
    fn from(string: &str) -> Self {
        // See if tag and remove # if so
        let is_tag = string.starts_with('#');
        let string = if is_tag { &string[1..] } else { string };

        // Parse the rest of the namespace
        let mut split = string.split(':').take(2);

        if !is_tag {
            match (split.next(), split.next()) {
                (Some(ns), Some(id)) => NamespacedId::Resource((ns.into(), id.into())),
                (Some(id), None) => NamespacedId::Resource(("minecraft".into(), id.into())),
                _ => NamespacedId::Invalid
            }
        } else {
            NamespacedId::Tag(string.into())
        }

    }
}

#[derive(Clone)]
pub struct ElementFaces {
    up: Option<NamespacedId>,
    down: Option<NamespacedId>,
    north: Option<NamespacedId>,
    east: Option<NamespacedId>,
    south: Option<NamespacedId>,
    west: Option<NamespacedId>
}

type ElementCorner = (f32, f32, f32);

#[derive(Clone)]
pub struct Element {
    pub from: ElementCorner,
    pub to: ElementCorner,
    pub face_textures: ElementFaces
}

//Deserialized info about a block and how it should render
pub struct BlockModelData {
    pub id: NamespacedId, //It's id
    pub parent: Option<NamespacedId>,
    pub elements: Vec<Element>,
    pub display_transforms: HashMap<String, Matrix4<f32>>,
    pub textures: HashMap<String, NamespacedId>
}

impl BlockModelData {
    fn triplet_from_array(vec: &Vec<Value>) -> Option<ElementCorner> {
        Option::Some((
            match vec.get(0).unwrap() {
                Value::Number(n) => {
                    n.as_f64()? as f32
                },
                _ => panic!("Invalid block datapack!")
            },
            match vec.get(1).unwrap() {
                Value::Number(n) => {
                    n.as_f64()? as f32
                },
                _ => panic!("Invalid block datapack!")
            },
            match vec.get(2).unwrap() {
                Value::Number(n) => {
                    n.as_f64()? as f32
                },
                _ => panic!("Invalid block datapack!")
            }
        ))
    }

    fn parse_elements(val: Option<&Value>, parent: Option<&BlockModelData>, textures: &HashMap<String, NamespacedId>) -> Option<Vec<Element>> {
        Option::Some(match val {
            None => match parent {
                None => Vec::new(),
                Some(parent) => {
                    parent.elements.clone()
                }
            },
            Some(v) => match v {
                Value::Array(arr) => {
                    let out = arr.iter().map(|x| {
                        let from = match x.get("from").unwrap() {
                            Value::Array(vec) => {
                                let triplet = BlockModelData::triplet_from_array(vec)?;

                                (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0)
                            },
                            _ => panic!("Invalid datapack!")
                        };

                        let to = match x.get("to").unwrap() {
                            Value::Array(vec) => {
                                let triplet = BlockModelData::triplet_from_array(vec)?;

                                (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0)
                            },
                            _ => panic!("Invalid datapack!")
                        };

                        Option::Some(Element {
                            from: (from.0, to.1, to.1),
                            to: (to.0, from.1, from.2),
                            face_textures: {
                                ElementFaces {
                                    up: None,
                                    down: None,
                                    north: None,
                                    east: None,
                                    south: None,
                                    west: None
                                }
                            }
                        })
                    }).collect::<Vec<Option<Element>>>();

                    for el in out.iter() {
                        match el {
                            None => return Option::None,
                            Some(_) => {}
                        }
                    }

                    out.into_iter().map(|x| x.unwrap()).collect()
                }, //TODO
                _ => Vec::new()
            }
        })
    }

    pub fn deserialize(name: &str, models_dir: PathBuf, model_map: &mut HashMap<String, BlockModelData>) {
        let path = models_dir.join(format!("{}.json", name));

        if model_map.contains_key(name) {
            return;
        }

        let bytes = std::fs::read(path).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let model = match json {
            Value::Object(obj) => {
                let (parent_model, parent_namespace) = {
                    match obj.get("parent") {
                        None => (Option::None, Option::None),
                        Some(v) => match v {
                            Value::String(s) => {
                                let namespaced = NamespacedId::from(&s[..]);

                                let namespace;
                                let id;

                                let path: &str = match &namespaced {
                                    NamespacedId::Resource(res) => {
                                        namespace = &res.0[..];
                                        id = &res.1[..];

                                        let r = &res.1;
                                        r.split(":").collect::<Vec<&str>>().last().unwrap().split("/").collect::<Vec<&str>>().last().unwrap()
                                    }
                                    _ => panic!("Invalid datapack!")
                                };

                                BlockModelData::deserialize(path, models_dir, model_map);

                                (
                                    model_map.get(&format!("{}:{}", namespace, id)),
                                    Option::Some(namespaced)
                                )
                            },
                            _ => panic!("Invalid datapack!")
                        }
                    }
                };

                let mut textures: HashMap<String, NamespacedId> = match obj.get("textures") {
                    None => HashMap::new(),
                    Some(textures_map) => {
                        let mut map: HashMap<String, NamespacedId> = textures_map.as_object().unwrap().iter().map(|(key, val)| {
                            (String::from(key), match val {
                                Value::String(str) => NamespacedId::from(&str[..]),
                                _ => panic!("Invalid datapack!")
                            })
                        }).collect();

                        match &parent_model {
                            None => map,
                            Some(p) => {
                                map.extend(
                                    p.textures.iter().map(|(k,v)| (k.clone(), v.clone()))
                                );
                                map
                            }
                        }
                    }
                };

                Option::Some(BlockModelData {
                    id: NamespacedId::Resource(("minecraft".into(), format!("block/{}", name))),
                    parent: parent_namespace,
                    elements: {
                        Self::parse_elements(
                            obj.get("elements"), parent_model.clone(), &textures
                        ).expect(name)
                    },
                    textures: {
                        textures
                    },
                    display_transforms: {
                        let map = HashMap::new();

                        map
                    }, //TODO
                    // textures: Option
                })
            },
            _ => Option::None,
        };

        match model {
            None => {}
            Some(m) => {
                // println!("Deserialized {:?} with {} elements", m.id, m.elements.len());
                match &m.id {
                    NamespacedId::Resource(namespace) => {
                        model_map.insert(format!("{}:{}", namespace.0, namespace.1), m);
                    },
                    _ => unreachable!()
                }
            }
        };
    }
}