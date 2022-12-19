use core::fmt::Debug;
use std::collections::HashMap;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::{mem, slice};

use jni::objects::{GlobalRef, JClass, JObject, JValue, ReleaseMode};
use jni::sys::{jbyteArray, jint, jlong, jlongArray, jobject};
use jni::JNIEnv;
use mc_varint::VarIntRead;

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

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createPalette(
    _env: JNIEnv,
    _class: JClass,
    idList: jlong,
) -> jlong {
    let palette = Box::new(JavaPalette::new(
        NonZeroUsize::new(idList as usize).unwrap(),
    ));

    Box::leak(palette) as *mut JavaPalette as usize as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_clearPalette(
    _env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
) {
    let palette = (palette_long as usize) as *mut JavaPalette;

    unsafe { palette.as_mut().unwrap().clear() };
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_destroyPalette(
    _env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
) {
    let palette = (palette_long as usize) as *mut JavaPalette;

    unsafe { Box::from_raw(palette) };
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteIndex(
    env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
    object: JObject,
    blockstate_index: jint,
) -> jint {
    let palette = (palette_long as usize) as *mut JavaPalette;

    (unsafe {
        palette.as_mut().unwrap().index((
            env.new_global_ref(object).unwrap(),
            (blockstate_index as u32).into(),
        ))
    }) as jint
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteSize(
    _env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
) -> jint {
    let palette = (palette_long as usize) as *mut JavaPalette;

    (unsafe { palette.as_ref().unwrap().size() }) as jint
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_copyPalette(
    _env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
) -> jlong {
    let palette = (palette_long as usize) as *mut JavaPalette;
    let mut new_palette = Box::new(unsafe { palette.as_ref().unwrap().clone() });
    let new_palette_ptr = &mut *new_palette as *mut JavaPalette;
    mem::forget(new_palette);

    new_palette_ptr as usize as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteGet(
    _env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
    index: i32,
) -> jobject {
    let palette = (palette_long as usize) as *mut JavaPalette;
    let palette = unsafe { palette.as_ref().expect("Palette pointer was null") };

    match palette.get(index as usize) {
        Some((global_ref, _)) => {
            return global_ref.as_obj().into_inner();
        }
        None => {
            panic!("Palette index {index} was not occupied\nPalette:\n{palette:?}");
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteReadPacket(
    env: JNIEnv,
    _class: JClass,
    palette_long: jlong,
    array: jbyteArray,
    current_position: jint,
    blockstate_offsets: jlongArray,
) -> jint {
    let palette = unsafe {
        ((palette_long as usize) as *mut JavaPalette)
            .as_mut()
            .unwrap()
    };
    let array = env
        .get_byte_array_elements(array, ReleaseMode::NoCopyBack)
        .unwrap();

    let blockstate_offsets_array = env
        .get_int_array_elements(blockstate_offsets, ReleaseMode::NoCopyBack)
        .unwrap();

    let id_list = unsafe { &*(palette.id_list.get() as *mut IdList) };

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

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_debugPalette(
    env: JNIEnv,
    _class: JClass,
    _packed_integer_array: jlong,
    palette: jlong,
) {
    // let array = unsafe { ((packed_integer_array as usize) as *mut PackedIntegerArray).as_ref().unwrap() };
    let palette = unsafe { ((palette as usize) as *mut JavaPalette).as_ref().unwrap() };

    palette.store.iter().for_each(|item| {
        env.call_static_method(
            "dev/birb/wgpu/render/Wgpu",
            "debug",
            "(Ljava/lang/Object;)V",
            &[JValue::Object(item.0.as_obj())],
        )
        .unwrap();
    });

    // let wm = RENDERER.get().unwrap();
    // let bm = wm.mc.block_manager.read();
    //
    // // println!("{:?}", palette.indices);
    //
    // (0..10).for_each(|id| {
    //     let key = array.get_by_index(id);
    //     match palette.get(key as usize) {
    //         Some((_, blockstate_key)) => {
    //             let (name, _) = bm.blocks.get_index(blockstate_key.block as usize).unwrap();
    //             println!("{}", name);
    //         },
    //         None => {},
    //     }
    // });
    // println!(
    //     "array val index: {} computed: {} ptr: {} raw read: {} val: {}\n{:?}",
    //     index,
    //     array.compute_storage_index(index),
    //     array.debug_pointer(index),
    //     unsafe { (array.debug_pointer(index) as *mut i64).read_volatile() },
    //     array.get_by_index(index),
    //     array
    // );
    // dbg!(array.index_offset, array.index_scale, array.index_shift, array.element_bits, array.size, array.element_bits, array.elements_per_long);
}
