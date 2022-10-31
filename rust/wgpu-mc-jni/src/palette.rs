use core::fmt::Debug;
use std::collections::HashMap;
use std::num::NonZeroUsize;

use jni::objects::GlobalRef;
use jni::sys::jobject;

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
    pub store: Vec<(GlobalRef, BlockstateKey)>,
    pub indices: HashMap<BlockstateKey, usize>,
    pub id_list: NonZeroUsize,
}

impl JavaPalette {
    pub fn new(id_list: NonZeroUsize) -> Self {
        Self {
            store: Vec::with_capacity(5),
            indices: HashMap::new(),
            id_list,
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

    #[allow(dead_code)]
    pub fn has_any(&self, predicate: &dyn Fn(jobject) -> bool) -> bool {
        self.store
            .iter()
            .any(|(global_ref, _)| predicate(global_ref.as_obj().into_inner()))
    }

    pub fn size(&self) -> usize {
        self.store.len()
    }

    pub fn get(&self, index: usize) -> Option<&(GlobalRef, BlockstateKey)> {
        self.store.get(index).or_else(|| self.store.get(0))
    }

    pub fn clear(&mut self) {
        self.store.clear();
        self.indices.clear();
    }
}

impl Debug for JavaPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = f.write_str("JavaPalette { store: [");
        self.store.iter().for_each(|store_entry| {
            write!(f, "(GlobalRef, {:?})", store_entry.1).unwrap();
        });
        res
    }
}
