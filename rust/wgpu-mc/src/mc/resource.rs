use crate::mc::datapack::NamespacedResource;

pub trait ResourceProvider: Send + Sync {
    fn get_resource(&self, id: &NamespacedResource) -> Vec<u8>;
}
