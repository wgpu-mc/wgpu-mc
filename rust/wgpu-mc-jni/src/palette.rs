use std::collections::HashMap;
use jni::objects::JObject;
use wgpu_mc::mc::block::PackedBlockstateKey;

#[derive(Clone)]
pub struct JavaPalette {
    store: Vec<*mut u8>,
    indices: HashMap<*mut u8, usize>
}

impl JavaPalette {
    
    pub fn new() -> Self {
        Self {
            store: Vec::new(),
            indices: HashMap::new()
        }
    }

    pub fn index(&mut self, object: JObject) -> usize {
        match self.indices.get(&(object.into_inner() as *mut u8)) {
            None => {
                self.store.push(object.into_inner() as *mut u8);
                self.store.len() - 1
            }
            Some(&index) => index
        }
    }

    pub fn has_any(&self, predicate: &dyn Fn(&*mut u8) -> bool) -> bool {
        self.store.iter().any(predicate)
    }

    pub fn size(&self) -> usize {
        self.store.len()
    }

    pub fn get(&self, index: usize) -> *mut u8 {
        self.store[index]
    }

}