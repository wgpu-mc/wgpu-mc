use crate::mc::datapack::{TextureVariableOrResource, NamespacedResource};
use std::sync::Arc;
use parking_lot::RwLock;

pub trait ResourceProvider: Send + Sync {
    fn get_resource(&self, id: &NamespacedResource) -> Vec<u8>;
}