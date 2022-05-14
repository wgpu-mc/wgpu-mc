use crate::mc::datapack::NamespacedResource;

///A trait used to get any form of resource from datapacks, such as json files or sound.
pub trait ResourceProvider: Send + Sync {
    fn get_resource(&self, id: &NamespacedResource) -> Option<Vec<u8>>;
}
