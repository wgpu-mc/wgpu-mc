use crate::mc::datapack::Identifier;

pub enum ResourceType {
    Texture,
}

pub trait ResourceProvider {
    fn get_bytes(&self, t: ResourceType, id: &Identifier) -> Vec<u8>;
}
