use crate::mc::datapack::{TagOrResource, NamespacedResource};
use std::sync::Arc;
use parking_lot::RwLock;

pub trait ResourceProvider: Send + Sync {

    fn get_resource(&self, id: &NamespacedResource) -> Vec<u8>;

}