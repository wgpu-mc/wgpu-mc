use std::collections::HashMap;

use crate::texture::UV;

use crate::mc::resource::ResourceProvider;
use cgmath::Matrix4;
use serde_json::Value;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

#[macro_export]
macro_rules! nsr (
    ($ns:literal) => {
        {
            use wgpu_mc::mc::datapack::NamespacedResource;
            NamespacedResource::try_from($ns).unwrap()
        }
    }
);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct NamespacedResource(pub String, pub String);

impl NamespacedResource {
    pub fn append(&self, a: &str) -> Self {
        Self(self.0.clone(), format!("{}{}", self.1, a))
    }

    pub fn prepend(&self, a: &str) -> Self {
        Self(self.0.clone(), format!("{}{}", a, self.1))
    }
}

impl Display for NamespacedResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
            .and(f.write_str(":").and(f.write_str(&self.1)))
    }
}

impl TryFrom<&str> for NamespacedResource {
    type Error = ();

    fn try_from(string: &str) -> Result<Self, Self::Error> {
        // Parse the rest of the namespace
        let mut split = string.split(':').take(2);

        Ok(match (split.next(), split.next()) {
            (Some(ns), Some(id)) => Self(ns.into(), id.into()),
            (Some(id), None) => Self("minecraft".into(), id.into()),
            _ => return Err(()),
        })
    }
}

impl From<(&str, &str)> for NamespacedResource {
    fn from(strings: (&str, &str)) -> Self {
        Self(strings.0.into(), strings.1.into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TextureVariableOrResource {
    Tag(String),
    Resource(NamespacedResource),
}

impl TextureVariableOrResource {
    #[must_use]
    pub fn is_tag(&self) -> bool {
        matches!(self, TextureVariableOrResource::Tag(_))
    }

    pub fn recurse_resolve_as_resource<'a, 'b: 'a>(
        &'a self,
        map: &'b HashMap<String, TextureVariableOrResource>,
    ) -> Option<&'a NamespacedResource> {
        match &self {
            TextureVariableOrResource::Tag(tag) => map.get(tag)?.recurse_resolve_as_resource(map),
            TextureVariableOrResource::Resource(resource) => Some(resource),
        }
    }

    pub fn as_resource(&self) -> Option<&NamespacedResource> {
        match self {
            TextureVariableOrResource::Tag(_) => None,
            TextureVariableOrResource::Resource(ref res) => Some(res),
        }
    }

    pub fn as_tag(&self) -> Option<&str> {
        match self {
            TextureVariableOrResource::Tag(string) => Some(string),
            TextureVariableOrResource::Resource(ref _res) => None,
        }
    }
}

impl Display for TextureVariableOrResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureVariableOrResource::Tag(tag) => f.write_str(&format!("#{}", tag)),
            TextureVariableOrResource::Resource(res) => {
                f.write_str(&format!("{}:{}", res.0, res.1))
            }
        }
    }
}

impl TryFrom<&str> for TextureVariableOrResource {
    type Error = ();

    fn try_from(string: &str) -> Result<TextureVariableOrResource, Self::Error> {
        // See if tag and remove # if so
        let is_tag = string.starts_with('#');
        let string = if is_tag { &string[1..] } else { string };

        // Parse the rest of the namespace
        let mut split = string.split(':').take(2);

        Ok(if !is_tag {
            match (split.next(), split.next()) {
                (Some(ns), Some(id)) => {
                    TextureVariableOrResource::Resource(NamespacedResource(ns.into(), id.into()))
                }
                (Some(id), None) => TextureVariableOrResource::Resource(NamespacedResource(
                    "minecraft".into(),
                    id.into(),
                )),
                _ => return Err(()),
            }
        } else {
            TextureVariableOrResource::Tag(string.into())
        })
    }
}

#[derive(Debug, Clone)]
pub struct FaceTexture {
    pub uv: UV,
    pub texture: TextureVariableOrResource,
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

///A struct that described a block and how it renders
/// Not a mesh! That would be BlockstateVariantMesh
#[derive(Clone, Debug)]
pub struct BlockModel {
    pub id: NamespacedResource,
    pub parent: Option<NamespacedResource>,
    pub elements: Vec<Element>,
    ///Depending on the camera state, e.g. 3rd or 1st person, the way the block is rendered is changed
    pub display_transforms: HashMap<String, Matrix4<f32>>,
    pub textures: HashMap<String, TextureVariableOrResource>,
}

impl BlockModel {
    fn triplet_from_array(vec: &[Value]) -> Option<ElementCorner> {
        Some((
            vec[0].as_f64()? as f32,
            vec[1].as_f64()? as f32,
            vec[2].as_f64()? as f32,
        ))
    }

    fn parse_face(
        face: Option<&Value>,
        _textures: &HashMap<String, TextureVariableOrResource>,
    ) -> Option<FaceTexture> {
        let face = face?.as_object()?;
        let uv = face.get("uv").map_or(((0.0, 0.0), (16.0, 16.0)), |uv| {
            let arr = uv.as_array().unwrap();
            (
                (
                    arr[0].as_f64().unwrap() as f32,
                    arr[1].as_f64().unwrap() as f32,
                ),
                (
                    arr[2].as_f64().unwrap() as f32,
                    arr[3].as_f64().unwrap() as f32,
                ),
            )
        });

        let texture: TextureVariableOrResource = face.get("texture")?.as_str()?.try_into().ok()?;

        Some(FaceTexture { uv, texture })
    }

    fn parse_elements(
        _debug: &NamespacedResource,
        val: Option<&Value>,
        parent: Option<&BlockModel>,
        textures: &HashMap<String, TextureVariableOrResource>,
    ) -> Option<Vec<Element>> {
        let _cobble =
            NamespacedResource("minecraft".into(), "models/block/cobblestone.json".into());

        match val {
            //No elements, default to parent's elements
            None => match parent {
                Some(parent) => Some(parent.elements.clone()),
                None => Some(Vec::new()),
            },
            Some(v) => {
                v.as_array()?
                    .iter()
                    .map(|element| {
                        let triplet = Self::triplet_from_array(element.get("from")?.as_array()?)?;
                        let from = (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0);

                        let triplet = Self::triplet_from_array(element.get("to")?.as_array()?)?;
                        let to = (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0);

                        let faces = element.get("faces")?.as_object()?;

                        // println!("{:?}", faces);

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
                    .collect::<Option<Vec<Element>>>()
            }
        }
    }

    pub fn deserialize<'a>(
        identifier: &NamespacedResource,
        resource_provider: &dyn ResourceProvider,
        model_map: &'a mut HashMap<NamespacedResource, BlockModel>,
    ) -> Option<&'a Self> {
        if model_map.contains_key(identifier) {
            return model_map.get(identifier);
        }

        let bytes_opt = resource_provider.get_resource(&identifier.prepend("models/").append(".json"));
        if bytes_opt.is_none() { return None; }

        let bytes = bytes_opt.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let obj = json.as_object()?;

        //Get information about the parent model, if this model has one
        let parent = obj.get("parent").and_then(|v| {
            let parent_identifier_string = v.as_str().unwrap();
            let parent_identifier = NamespacedResource::try_from(parent_identifier_string).unwrap();

            BlockModel::deserialize(&parent_identifier, resource_provider, model_map).unwrap();

            model_map.get(&parent_identifier)
        });

        let this_textures: HashMap<String, TextureVariableOrResource> = json
            .get("textures")
            .map(|texture_val: &Value| {
                let val = texture_val.as_object().unwrap();
                val.iter()
                    .map(|(key, value)| {
                        (
                            key.clone(),
                            TextureVariableOrResource::try_from(value.as_str().unwrap()).unwrap(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut resolved_parent_textures: HashMap<String, TextureVariableOrResource> = match parent
        {
            None => HashMap::new(),
            Some(parent_model) => {
                let mut textures = parent_model.textures.clone();

                textures
                    .iter_mut()
                    .for_each(|(_key, value)| match value.clone() {
                        TextureVariableOrResource::Tag(tag_key) => {
                            match this_textures.get(&tag_key) {
                                None => {}
                                Some(resolved) => *value = resolved.clone(),
                            }
                        }
                        TextureVariableOrResource::Resource(_) => {}
                    });

                textures
            }
        };

        resolved_parent_textures.extend(this_textures.into_iter());

        let model = BlockModel {
            id: identifier.clone(),
            parent: parent.map(|some| some.id.clone()),
            elements: {
                Self::parse_elements(
                    identifier,
                    obj.get("elements"),
                    parent,
                    &resolved_parent_textures,
                )?
            },
            textures: resolved_parent_textures,
            display_transforms: HashMap::new(), //TODO
        };

        model_map.insert(identifier.clone(), model);

        Some(model_map.get(identifier).unwrap())
    }
}

pub struct AnimationData {
    pub frame_time: u32,
    pub interpolate: bool,
    pub frames: Option<Vec<u32>>,
}

impl AnimationData {
    pub fn new() -> Self {
        Self {
            frame_time: 0,
            interpolate: false,
            frames: Option::None,
        }
    }

    pub fn clone(animation: &AnimationData) -> Self {
        Self {
            frame_time: animation.frame_time,
            interpolate: animation.interpolate,
            frames: match &animation.frames {
                None => None,
                Some(frames) => {
                    let mut out = Vec::new();

                    for frame in frames {
                        out.push(*frame);
                    }

                    Some(out)
                }
            },
        }
    }

    pub fn deserialize(
        identifier: &NamespacedResource,
        resource_provider: &dyn ResourceProvider,
    ) -> Option<Self> {
        let res = identifier.append(".mcmeta");
        let optional_res = resource_provider.get_resource(&res);

        if optional_res.is_none() { return None; }

        let bytes = optional_res.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let obj = json.as_object().unwrap();

        let animation_opt = obj.get("animation");

        if animation_opt.is_none() { return None; }

        let animation = animation_opt.unwrap();

        Some(
            Self {
                frame_time: match animation.get("frametime") {
                    None => 1,
                    Some(f) => f.as_u64().unwrap() as u32
                },
                interpolate: match animation.get("interpolate") {
                    None => false,
                    Some(i) => i.as_bool().unwrap()
                },
                frames: match animation.get("frames") {
                    None => None,
                    Some(f) => {
                        let mut v : Vec<u32> = Vec::new();
                        for frame in f.as_array().unwrap().iter() {
                            v.push(frame.as_u64().unwrap() as u32);
                        }
                        Some(v)
                    }
                }
            }
        )
    }
}
