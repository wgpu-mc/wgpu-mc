use core::fmt::Debug;
use std::collections::HashMap;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::slice;

use jni::objects::{GlobalRef, JClass, JObject, JValue, ReleaseMode};
use jni::sys::{jbyteArray, jint, jlong, jlongArray, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use mc_varint::VarIntRead;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use slab::Slab;
use wgpu_mc::mc::Block;

use wgpu_mc::mc::block::BlockstateKey;
use wgpu_mc::WmRenderer;

pub static PALETTE_STORAGE: Lazy<RwLock<Slab<JavaPalette>>> =
    Lazy::new(|| RwLock::new(Slab::with_capacity(4096)));

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

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugPalette {
    pub store: Vec<String>,
    pub indices: HashMap<String, usize>,
}

impl DebugPalette {
    pub fn from_java_palette(wm: &WmRenderer, java_palette: &JavaPalette) -> Self {
        let block_manager = wm.mc.block_manager.read();

        let indices = java_palette
            .indices
            .iter()
            .map(|(id, index)| {
                let (block_name, block) =
                    block_manager.blocks.get_index(id.block as usize).unwrap();

                match block {
                    Block::Multipart(multipart) => {
                        let read = multipart.keys.read();
                        let (variant_name, _) = read.get_index(id.augment as usize).unwrap();
                        (format!("{block_name}${variant_name}"), *index)
                    }
                    Block::Variants(map) => {
                        let (variant_name, _) = map.get_index(id.augment as usize).unwrap();
                        (format!("{block_name}${variant_name}"), *index)
                    }
                }
            })
            .collect();

        let store = java_palette
            .store
            .iter()
            .map(|(_, id)| {
                let (block_name, block) =
                    block_manager.blocks.get_index(id.block as usize).unwrap();

                match block {
                    Block::Multipart(multipart) => {
                        let read = multipart.keys.read();
                        let (variant_name, _) = read.get_index(id.augment as usize).unwrap();
                        format!("{block_name}${variant_name}")
                    }
                    Block::Variants(map) => {
                        let (variant_name, _) = map.get_index(id.augment as usize).unwrap();
                        format!("{block_name}${variant_name}")
                    }
                }
            })
            .collect();

        Self { store, indices }
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

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createPalette(_env: JNIEnv, _class: JClass, idList: jlong) -> jlong {
    let palette = JavaPalette::new(NonZeroUsize::new(idList as usize).unwrap());
    PALETTE_STORAGE.write().insert(palette) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn clearPalette(_env: JNIEnv, _class: JClass, palette_long: jlong) {
    let mut storage_access = PALETTE_STORAGE.write();
    let palette = storage_access.get_mut(palette_long as usize).unwrap();
    palette.clear();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn destroyPalette(_env: JNIEnv, _class: JClass, palette_long: jlong) {
    panic!();
    PALETTE_STORAGE.write().remove(palette_long as usize);
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
    palette.index((
        env.new_global_ref(object).unwrap(),
        (blockstate_index as u32).into(),
    )) as jint
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
pub fn copyPalette(_env: JNIEnv, _class: JClass, palette_long: jlong) -> jlong {
    let mut storage_access = PALETTE_STORAGE.write();
    let palette = storage_access.get(palette_long as usize).unwrap();
    let new_palette = palette.clone();
    storage_access.insert(new_palette) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn paletteGet(_env: JNIEnv, _class: JClass, palette_long: jlong, index: i32) -> jobject {
    let storage_access = PALETTE_STORAGE.read();
    let palette = storage_access.get(palette_long as usize).unwrap();

    match palette.get(index as usize) {
        Some((global_ref, _)) => {
            return global_ref.as_obj().into_raw();
        }
        None => {
            panic!("Palette index {index} was not occupied\nPalette:\n{palette:?}");
        }
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn paletteReadPacket(
    env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
    array: jbyteArray,
    current_position: jint,
    blockstate_offsets: jlongArray,
) -> jint {
    let mut storage_access = PALETTE_STORAGE.write();
    let palette = storage_access.get_mut(palette_long as usize).unwrap();
    let array = env
        .get_byte_array_elements(array, ReleaseMode::NoCopyBack)
        .unwrap();

    let blockstate_offsets_array = env
        .get_int_array_elements(blockstate_offsets, ReleaseMode::NoCopyBack)
        .unwrap();

    let id_list = unsafe { &*(palette.id_list.get() as *const IdList) };

    let blockstate_offsets = unsafe {
        slice::from_raw_parts(
            blockstate_offsets_array.as_ptr() as *mut i32,
            blockstate_offsets_array.size().unwrap() as usize,
        )
    };

    let vec = unsafe {
        slice::from_raw_parts(
            array.as_ptr().offset(current_position as isize) as *mut u8,
            (array.size().unwrap() - current_position) as usize,
        )
    };

    let mut cursor = Cursor::new(vec);
    let packet_len: i32 = cursor.read_var_int().unwrap().into();

    for blockstate_offset in blockstate_offsets.iter().take(packet_len as usize) {
        let var_int: i32 = cursor.read_var_int().unwrap().into();

        let object = id_list.map.get(&var_int).unwrap().clone();

        palette.add((
            object,
            BlockstateKey {
                block: (blockstate_offset >> 16) as u16,
                augment: (blockstate_offset & 0xffff) as u16,
            },
        ));
    }

    //The amount of bytes read
    cursor.position() as jint
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn debugPalette(env: JNIEnv, _class: JClass, _packed_integer_array: jlong, palette: jlong) {
    let storage_access = PALETTE_STORAGE.read();
    let palette = storage_access.get(palette as usize).unwrap();
    palette.store.iter().for_each(|item| {
        env.call_static_method(
            "dev/birb/wgpu/render/Wgpu",
            "debug",
            "(Ljava/lang/Object;)V",
            &[JValue::Object(item.0.as_obj())],
        )
        .unwrap();
    });
}
