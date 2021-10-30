use std::collections::HashMap;
use std::path::PathBuf;

use crate::texture::UV;

use cgmath::{Matrix4, Vector2};
use serde_json::Value;

pub type NamespacedResource = (String, String);

#[derive(Debug, Clone, Eq, Hash)]
pub enum NamespacedId {
    Tag(String),
    Resource(NamespacedResource),
    Invalid,
}

impl NamespacedId {
    pub fn is_tag(&self) -> bool {
        matches!(self, NamespacedId::Tag(_))
    }
}

impl std::string::ToString for NamespacedId {
    fn to_string(&self) -> String {
        match self {
            NamespacedId::Tag(tag) => format!("#{}", tag),
            NamespacedId::Resource(res) => format!("{}:{}", res.0, res.1),
            NamespacedId::Invalid => "Invalid".into(),
        }
    }
}

impl PartialEq for NamespacedId {
    fn eq(&self, other: &Self) -> bool {
        match self {
            NamespacedId::Tag(tag) => {
                if let NamespacedId::Tag(o) = other {
                    o == tag
                } else {
                    false
                }
            }
            NamespacedId::Resource((ns, id)) => {
                if let NamespacedId::Resource((ons, oid)) = other {
                    ons == ns && oid == id
                } else {
                    false
                }
            }
            NamespacedId::Invalid => false,
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
                _ => NamespacedId::Invalid,
            }
        } else {
            NamespacedId::Tag(string.into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaceTexture {
    pub uv: UV,
    pub texture: NamespacedId,
}

#[derive(Debug, Clone)]
pub struct ElementFaces {
    pub up: Option<FaceTexture>,
    pub down: Option<FaceTexture>,
    pub north: Option<FaceTexture>,
    pub east: Option<FaceTexture>,
    pub south: Option<FaceTexture>,
    pub west: Option<FaceTexture>,
}

type ElementCorner = (f32, f32, f32);

#[derive(Debug, Clone)]
pub struct Element {
    pub from: ElementCorner,
    pub to: ElementCorner,
    pub face_textures: ElementFaces,
}

//Deserialized info about a block and how it should render
pub struct BlockModelData {
    pub id: NamespacedId, //Its id
    pub parent: Option<NamespacedId>,
    pub elements: Vec<Element>,
    pub display_transforms: HashMap<String, Matrix4<f32>>,
    pub textures: HashMap<String, NamespacedId>,
}

impl BlockModelData {
    fn triplet_from_array(vec: &[Value]) -> Option<ElementCorner> {
        Some((
            if let Value::Number(ref n) = vec[0] {
                n.as_f64()? as f32
            } else {
                panic!("Invalid block datapack!")
            },
            if let Value::Number(ref n) = vec[1] {
                n.as_f64()? as f32
            } else {
                panic!("Invalid block datapack!")
            },
            if let Value::Number(ref n) = vec[2] {
                n.as_f64()? as f32
            } else {
                panic!("Invalid block datapack!")
            },
        ))
    }

    #[allow(unused_variables)] // TODO parameter textures is unused
    fn parse_face(
        val: Option<&Value>,
        textures: &HashMap<String, NamespacedId>,
    ) -> Option<FaceTexture> {
        match val {
            None => None,
            Some(face) => {
                let obj = face.as_object().unwrap();
                let uv = match obj.get("uv") {
                    None => (Vector2::new(0.0, 0.0), Vector2::new(16.0, 16.0)),
                    Some(uv_arr_v) => {
                        let uv_arr = uv_arr_v.as_array().unwrap();
                        (
                            //TODO: handle UV rotation
                            Vector2::new(
                                uv_arr[0].as_f64().unwrap() as f32,
                                uv_arr[1].as_f64().unwrap() as f32,
                            ),
                            Vector2::new(
                                uv_arr[2].as_f64().unwrap() as f32,
                                uv_arr[3].as_f64().unwrap() as f32,
                            ),
                        )
                    }
                };

                let texture = NamespacedId::from(obj.get("texture").unwrap().as_str().unwrap());

                Some(FaceTexture { uv, texture })
            }
        }
    }

    fn parse_elements(
        val: Option<&Value>,
        parent: Option<&BlockModelData>,
        textures: &HashMap<String, NamespacedId>,
    ) -> Option<Vec<Element>> {
        Some(match val {
            //No elements, default to parent's
            None => match parent {
                Some(parent) => parent.elements.clone(),
                None => Vec::new(),
            },
            Some(v) => match v {
                //The array of elements
                Value::Array(arr) => {
                    let out: Vec<_> = arr
                        .iter()
                        .map(|x| {
                            let from = match x.get("from").unwrap() {
                                Value::Array(vec) => {
                                    let triplet = BlockModelData::triplet_from_array(vec)?;

                                    (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0)
                                }
                                _ => panic!("Invalid datapack!"),
                            };

                            let to = match x.get("to").unwrap() {
                                Value::Array(vec) => {
                                    let triplet = BlockModelData::triplet_from_array(vec)?;

                                    (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0)
                                }
                                _ => panic!("Invalid datapack!"),
                            };

                            let faces = x.get("faces").unwrap().as_object().unwrap();

                            Some(Element {
                                from,
                                to,
                                face_textures: {
                                    ElementFaces {
                                        up: Self::parse_face(faces.get("up"), textures),
                                        down: Self::parse_face(faces.get("down"), textures),
                                        north: Self::parse_face(faces.get("north"), textures),
                                        east: Self::parse_face(faces.get("east"), textures),
                                        south: Self::parse_face(faces.get("south"), textures),
                                        west: Self::parse_face(faces.get("west"), textures),
                                    }
                                },
                            })
                        })
                        .collect();

                    if out.iter().any(|x| x.is_none()) {
                        return None;
                    }

                    out.into_iter().map(|x| x.unwrap()).collect()
                } //TODO
                _ => Vec::new(),
            },
        })
    }

    pub fn deserialize(
        name: &str,
        models_dir: PathBuf,
        model_map: &mut HashMap<String, BlockModelData>,
    ) {
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
                        None => (None, None),
                        Some(v) => match v {
                            Value::String(s) => {
                                let namespaced = NamespacedId::from(s.as_str());

                                let namespace;
                                let id;

                                let path: &str = match &namespaced {
                                    NamespacedId::Resource(res) => {
                                        namespace = res.0.as_str();
                                        id = res.1.as_str();

                                        res.1
                                            .as_str()
                                            .split(':')
                                            .last()
                                            .unwrap()
                                            .split('/')
                                            .last()
                                            .unwrap()
                                    }
                                    _ => panic!("Invalid datapack!"),
                                };

                                BlockModelData::deserialize(path, models_dir, model_map);

                                (
                                    model_map.get(&format!("{}:{}", namespace, id)),
                                    Some(namespaced),
                                )
                            }
                            _ => panic!("Invalid datapack!"),
                        },
                    }
                };

                let textures: HashMap<String, NamespacedId> = match obj.get("textures") {
                    None => HashMap::new(),
                    Some(textures_map) => {
                        let mut map: HashMap<_, _> = textures_map
                            .as_object()
                            .unwrap()
                            .iter()
                            .map(|(key, val)| {
                                (
                                    key.clone(),
                                    match val {
                                        Value::String(str) => NamespacedId::from(&str[..]),
                                        _ => panic!("Invalid datapack!"),
                                    },
                                )
                            })
                            .collect();

                        match &parent_model {
                            None => map,
                            Some(p) => {
                                map.extend(p.textures.iter().map(|(k, v)| (k.clone(), v.clone())));
                                map
                            }
                        }
                    }
                };

                Some(BlockModelData {
                    id: NamespacedId::Resource(("minecraft".into(), format!("block/{}", name))),
                    parent: parent_namespace,
                    elements: {
                        Self::parse_elements(obj.get("elements"), parent_model, &textures)
                            .expect(name)
                    },
                    textures,
                    display_transforms: HashMap::new(), //TODO
                                                        // textures: Option
                })
            }
            _ => None,
        };

        if let Some(m) = model {
            // println!("Deserialized {:?} with {} elements", m.id, m.elements.len());
            if let NamespacedId::Resource(ref namespace) = m.id {
                model_map.insert(format!("{}:{}", namespace.0, namespace.1), m);
            }
        };
    }
}
