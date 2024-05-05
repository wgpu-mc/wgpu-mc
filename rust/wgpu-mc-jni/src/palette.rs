use core::fmt::Debug;
use std::collections::HashMap;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::slice;

use jni::objects::{GlobalRef, JByteArray, JClass, JLongArray, JObject, JValue, ReleaseMode};
use jni::sys::{jint, jlong, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use mc_varint::VarIntRead;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use slab::Slab;

use wgpu_mc::mc::block::BlockstateKey;

pub static PALETTE_STORAGE: Lazy<RwLock<Slab<JavaPalette>>> =
    Lazy::new(|| RwLock::new(Slab::with_capacity(4096)));

#[derive(Clone)]
pub struct JavaPalette {
    pub store: Vec<BlockstateKey>,
    pub indices: HashMap<BlockstateKey, usize>,
}
impl JavaPalette {
    pub fn new() -> Self {
        Self {
            store: Vec::with_capacity(5),
            indices: HashMap::new(),
        }
    }

    pub fn index(&mut self, element: BlockstateKey) -> usize {
        match self.indices.get(&element) {
            None => {
                self.indices.insert(element, self.store.len());
                self.store.push(element);
                self.store.len() - 1
            }
            Some(&index) => index,
        }
    }

    pub fn add(&mut self, element: BlockstateKey) {
        self.indices.insert(element, self.store.len());
        self.store.push(element);
    }

    pub fn size(&self) -> usize {
        self.store.len()
    }

    pub fn get(&self, index: usize) -> Option<&BlockstateKey> {
        self.store.get(index).or_else(|| self.store.first())
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
            write!(f, "(GlobalRef, {:?})", store_entry).unwrap();
        });
        res
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createPalette(_env: JNIEnv, _class: JClass) -> jlong {
    let palette = JavaPalette::new();
    PALETTE_STORAGE.write().insert(palette) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn clearPalette(_env: JNIEnv, _class: JClass, palette_long: jlong) {
    let mut storage_access = PALETTE_STORAGE.write();
    let palette = storage_access.get_mut(palette_long as usize).unwrap();
    palette.clear();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn destroyPalette(_env: JNIEnv, _class: JClass, _palette_long: jlong) {
    panic!();
    // PALETTE_STORAGE.write().remove(palette_long as usize);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn paletteIndex(
    env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
    object: JObject,
    blockstate_index: jint,
) -> jint {
    let mut storage_access = PALETTE_STORAGE.write();
    let palette = storage_access.get_mut(palette_long as usize).unwrap();
    palette.index((blockstate_index as u32).into()) as jint
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn paletteSize(_env: JNIEnv, _class: JClass, palette_long: jlong) -> jint {
    PALETTE_STORAGE
        .read()
        .get(palette_long as usize)
        .unwrap()
        .size() as jint
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn paletteReadPacket(
    mut env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
    array: JByteArray,
    current_position: jint,
    blockstate_offsets: JLongArray,
) -> jint {
    let mut storage_access = PALETTE_STORAGE.write();
    let palette = storage_access.get_mut(palette_long as usize).unwrap();
    let array = unsafe { env.get_array_elements(&array, ReleaseMode::NoCopyBack) }.unwrap();
    let blockstate_offsets_elements =
        unsafe { env.get_array_elements(&blockstate_offsets, ReleaseMode::NoCopyBack) }.unwrap();
    let blockstate_offsets = unsafe {
        slice::from_raw_parts(
            blockstate_offsets_elements.as_ptr(),
            blockstate_offsets_elements.len(),
        )
    };

    let vec = unsafe {
        slice::from_raw_parts(
            array.as_ptr().offset(current_position as isize) as *mut u8,
            array.len() - current_position as usize,
        )
    };

    let mut cursor = Cursor::new(vec);
    let packet_len: i32 = cursor.read_var_int().unwrap().into();

    for blockstate_offset in blockstate_offsets.iter().take(packet_len as usize) {
        let var_int: i32 = cursor.read_var_int().unwrap().into();

        palette.add(BlockstateKey {
            block: (blockstate_offset >> 16) as u16,
            augment: (blockstate_offset & 0xffff) as u16,
        });
    }

    //The amount of bytes read
    cursor.position() as jint
}
