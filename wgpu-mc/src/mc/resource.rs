use crate::mc::datapack::Identifier;
use std::sync::Arc;
use parking_lot::RwLock;

pub trait ResourceProvider: Send + Sync {

    fn get_resource(&self, id: &Identifier) -> &[u8];

}