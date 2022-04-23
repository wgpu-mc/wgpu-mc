use std::collections::HashMap;
use jni::objects::{GlobalRef, JObject};
use jni::sys::{jlong, jobject};
use wgpu_mc::mc::block::PackedBlockstateKey;

pub struct IdList {
    pub map: HashMap<i32, GlobalRef>,
}

impl IdList {

    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

}

#[derive(Clone)]
pub struct JavaPalette {
    store: Vec<GlobalRef>,
    indices: HashMap<jobject, usize>,
    pub id_list: *mut IdList
}

impl JavaPalette {
    
    pub fn new(id_list: jlong) -> Self {
        Self {
            store: Vec::new(),
            indices: HashMap::new(),
            id_list: id_list as usize as *mut IdList
        }
    }

    pub fn index(&mut self, object: GlobalRef) -> usize {
        match self.indices.get(&object.as_obj().into_inner()) {
            None => {
                self.indices.insert(object.as_obj().into_inner(), self.store.len());
                self.store.push(object);
                self.store.len() - 1
            }
            Some(&index) => index
        }
    }

    pub fn add(&mut self, object: GlobalRef) {
        self.indices.insert(object.as_obj().into_inner(), self.store.len());
        self.store.push(object);
    }

    pub fn has_any(&self, predicate: &dyn Fn(jobject) -> bool) -> bool {
        self.store.iter().any(|global_ref| predicate(global_ref.as_obj().into_inner()))
    }

    pub fn size(&self) -> usize {
        self.store.len()
    }

    pub fn get(&self, index: usize) -> jobject {
        self.store[index].as_obj().into_inner()
    }

}