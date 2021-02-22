use crate::mc::datapack::NamespacedId;

pub enum ResourceType {
    Texture
}

pub trait ResourceProvider {

    fn get_bytes(&self, t: ResourceType, id: &NamespacedId) -> Vec<u8>;

}