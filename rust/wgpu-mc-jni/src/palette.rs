use jni::objects::{GlobalRef, JObject};
use jni::sys::{jlong, jobject};
use wgpu_mc::mc::Block;
use std::collections::HashMap;
use wgpu_mc::mc::block::BlockstateKey;

pub struct IdList {
    pub map: HashMap<i32, GlobalRef>,
}

impl IdList {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct JavaPalette {
    store: Vec<(GlobalRef, BlockstateKey)>,
    indices: HashMap<BlockstateKey, usize>,
    pub id_list: *mut IdList,
}

impl JavaPalette {
    pub fn new(id_list: jlong) -> Self {
        Self {
            store: Vec::with_capacity(256),
            indices: HashMap::new(),
            id_list: id_list as usize as *mut IdList,
        }
    }

    pub fn index(&mut self, element: (GlobalRef, BlockstateKey)) -> usize {
        match self.indices.get(&element.1) {
            None => {
                self.indices.insert(element.1, self.store.len());
                self.store.push(element);
                self.store.len() - 1
            }
            Some(&index) => index,
        }
    }

    pub fn add(&mut self, element: (GlobalRef, BlockstateKey)) {
        self.indices.insert(element.1, self.store.len());
        self.store.push(element);
    }

    pub fn has_any(&self, predicate: &dyn Fn(jobject) -> bool) -> bool {
        self.store
            .iter()
            .any(|(global_ref, _)| predicate(global_ref.as_obj().into_inner()))
    }

    pub fn size(&self) -> usize {
        self.store.len()
    }

    pub fn get(&self, index: usize) -> Option<&(GlobalRef, BlockstateKey)> {
        self.store.get(index)
    }

    pub fn clear(&mut self) {
        self.store.clear();
        self.indices.clear();
    }
}
