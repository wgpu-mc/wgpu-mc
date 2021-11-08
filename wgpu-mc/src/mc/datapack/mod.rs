use std::collections::HashMap;
use std::path::PathBuf;

use crate::texture::UV;

use cgmath::{Matrix4, Vector2};
use serde_json::Value;
use std::convert::{TryFrom, TryInto};
use crate::mc::resource::ResourceProvider;

pub type NamespacedResource = (String, String);

///TODO: make this be a struct that contains only NamespacedResource and no Tag
#[derive(Debug, Clone, Eq, Hash)]
pub enum Identifier {
    Tag(String),
    Resource(NamespacedResource)
}

impl Identifier {
    pub fn is_tag(&self) -> bool {
        matches!(self, Identifier::Tag(_))
    }
}

impl std::string::ToString for Identifier {
    fn to_string(&self) -> String {
        match self {
            Identifier::Tag(tag) => format!("#{}", tag),
            Identifier::Resource(res) => format!("{}:{}", res.0, res.1)
        }
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Identifier::Tag(tag) => {
                if let Identifier::Tag(o) = other {
                    o == tag
                } else {
                    false
                }
            }
            Identifier::Resource((ns, id)) => {
                if let Identifier::Resource((ons, oid)) = other {
                    ons == ns && oid == id
                } else {
                    false
                }
            }
        }
    }
}

impl TryFrom<&str> for Identifier {
    type Error = ();

    fn try_from(string: &str) -> Result<Identifier, Self::Error> {
        // See if tag and remove # if so
        let is_tag = string.starts_with('#');
        let string = if is_tag { &string[1..] } else { string };

        // Parse the rest of the namespace
        let mut split = string.split(':').take(2);

        Ok(if !is_tag {
            match (split.next(), split.next()) {
                (Some(ns), Some(id)) => Identifier::Resource((ns.into(), id.into())),
                (Some(id), None) => Identifier::Resource(("minecraft".into(), id.into())),
                _ => return Err(())
            }
        } else {
            Identifier::Tag(string.into())
        })
    }
}

#[derive(Debug, Clone)]
pub struct FaceTexture {
    pub uv: UV,
    pub texture: Identifier,
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

///A struct that described a block and how it renders (usually from a datapack)
/// Not a baked mesh.
#[derive(Clone, Debug)]
pub struct BlockModel {
    pub id: Identifier, //Its id
    pub parent: Option<Identifier>,
    pub elements: Vec<Element>,
    pub display_transforms: HashMap<String, Matrix4<f32>>,
    pub textures: HashMap<String, Identifier>,
}

impl BlockModel {
    fn triplet_from_array(vec: &[Value]) -> Option<ElementCorner> {

        Some(
            (
                vec[0].as_f64()? as f32,
                vec[0].as_f64()? as f32,
                vec[0].as_f64()? as f32
            )
        )
    }

    fn parse_face(
        val: Option<&Value>
    ) -> Option<FaceTexture> {
        let face = val?.as_object()?;
        let uv_arr = face.get("uv")?.as_array()?;

        let uv = (
            //TODO: handle UV rotation
            (
                uv_arr[0].as_f64().unwrap() as f32,
                uv_arr[1].as_f64().unwrap() as f32,
            ),
            (
                uv_arr[2].as_f64().unwrap() as f32,
                uv_arr[3].as_f64().unwrap() as f32,
            ),
        );

        let texture: Identifier = face.get("texture")?.as_str()?.try_into().ok()?;

        Some(FaceTexture {
            uv,
            texture
        })
    }

    fn parse_elements(
        val: Option<&Value>,
        parent: Option<&BlockModel>,
        textures: &HashMap<String, Identifier>,
    ) -> Option<Vec<Element>> {
        match val {
            //No elements, default to parent's elements
            None => match parent {
                Some(parent) => Some(parent.elements.clone()),
                None => Some(Vec::new()),
            },
            Some(v) => {
                val?.as_array()?
                    .iter()
                    .map(|element| {
                        let triplet = Self::triplet_from_array(element.get("from")?.as_array()?)?;
                        let from = (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0);

                        let triplet = Self::triplet_from_array(element.get("to")?.as_array()?)?;
                        let to = (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0);

                        let faces = element.get("faces")?.as_object()?;

                        Some(Element {
                            from,
                            to,
                            face_textures: {
                                ElementFaces {
                                    up: Self::parse_face(faces.get("up")),
                                    down: Self::parse_face(faces.get("down")),
                                    north: Self::parse_face(faces.get("north")),
                                    east: Self::parse_face(faces.get("east")),
                                    south: Self::parse_face(faces.get("south")),
                                    west: Self::parse_face(faces.get("west")),
                                }
                            },
                        })
                    }).collect::<Option<Vec<Element>>>()
            }
        }
    }

    pub fn deserialize(
        identifier: &Identifier,
        resource_provider: &dyn ResourceProvider,
        model_map: &mut HashMap<Identifier, BlockModel>,
    ) -> Option<()> {
        if model_map.contains_key(identifier) {
            return Some(());
        }

        let bytes = resource_provider.get_resource(identifier);
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let obj = json.as_object()?;

        //Get information about the parent model, if this model has one
        let parent = obj.get("parent").and_then(|v| {
            let parent_identifier_string = v.as_str()?;
            let parent_identifier: Identifier = parent_identifier_string.try_into().unwrap();

            BlockModel::deserialize(&parent_identifier, resource_provider, model_map);

            model_map.get(&parent_identifier)
        });

        //Get the face texture mappings
        let mut textures: HashMap<String, Identifier> = obj.get("textures").map_or(
            HashMap::new(),
            |textures_map| {
                //Map of the faces and their textures
                let mut map: HashMap<String, Identifier> = textures_map
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(key, val)| {
                        (
                            key.clone(),
                            val.as_str().unwrap().try_into().unwrap()
                        )
                    })
                    .collect();

                //If there is a parent model, merge the texture references so that the tags can be resolved.
                match parent {
                    None => map,
                    Some(parent_model) => {
                        map.extend(parent_model.textures.iter().map(|(k, v)| (k.clone(), v.clone())));
                        map
                    }
                }
            }
        );

        let resolved_resources: HashMap<String, Identifier> = textures.clone().into_iter().filter(|(string, identifier)| {
            matches!(identifier, Identifier::Resource(_))
        }).collect();

        textures.values_mut().for_each(|identifier| {
            if let Identifier::Tag(tag) = identifier {
                *identifier = resolved_resources.get(tag).unwrap().clone();
            }
        });

        let model = BlockModel {
            id: identifier.clone(),
            parent: parent.map(|some| {
                some.id.clone()
            }),
            elements: {
                Self::parse_elements(obj.get("elements"), parent, &textures)?
            },
            textures,
            display_transforms: HashMap::new(), //TODO
        };

        model_map.insert(identifier.clone(), model);

        Some(())
    }
}
