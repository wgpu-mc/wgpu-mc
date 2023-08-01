use std::slice;

use jni::objects::{JClass, JLongArray, ReleaseMode};
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use slab::Slab;

pub static PIA_STORAGE: Lazy<RwLock<Slab<PackedIntegerArray>>> =
    Lazy::new(|| RwLock::new(Slab::with_capacity(2048)));

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackedIntegerArray {
    data: Box<[i64]>,
    elements_per_long: i32,
    element_bits: i32,
    max_value: i64,
    index_scale: i32,
    index_offset: i32,
    index_shift: i32,
    size: i32,
}

impl PackedIntegerArray {
    pub fn get(&self, x: i32, y: i32, z: i32) -> i32 {
        let x = x & 0xf;
        let y = y & 0xf;
        let z = z & 0xf;

        self.get_by_index((((y << 4) | z) << 4) | x)
    }

    pub fn get_by_index(&self, index: i32) -> i32 {
        assert!(index < self.size, "index: {}, size: {}", index, self.size);

        let i: i32 = self.compute_storage_index(index);

        let l: i64 = self.data[i as usize];

        let j: i32 = (index - (i * self.elements_per_long)) * self.element_bits;
        ((l >> j) & self.max_value) as i32
    }

    pub fn compute_storage_index(&self, index: i32) -> i32 {
        let l = self.index_scale as u32 as i64;
        let m = self.index_offset as u32 as i64;
        (((((index as i64) * l) + m) >> 32) >> self.index_shift) as i32
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createPaletteStorage(
    mut env: JNIEnv,
    _class: JClass,
    data: JLongArray,
    elements_per_long: jint,
    element_bits: jint,
    max_value: jlong,
    index_scale: jint,
    index_offset: jint,
    index_shift: jint,
    size: jint,
) -> jlong {
    // let copy = env
    //     .get_long_array_elements(data, ReleaseMode::NoCopyBack)
    //     .unwrap();

    let copy = unsafe { env.get_array_elements_critical(&data, ReleaseMode::NoCopyBack) }.unwrap();

    let packed_arr = PackedIntegerArray {
        data: Vec::from(unsafe { slice::from_raw_parts(copy.as_ptr() as *mut jlong, copy.len()) })
            .into_boxed_slice(),
        elements_per_long,
        element_bits,
        max_value,
        index_scale,
        index_offset,
        index_shift,
        size,
    };

    let mut storage = PIA_STORAGE.write();
    storage.insert(packed_arr) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn piaGet(_env: JNIEnv, _class: JClass, pia: jlong, x: jint, y: jint, z: jint) -> jint {
    let storage = PIA_STORAGE.read();
    let pia = storage.get(pia as usize).unwrap();
    pia.get(x, y, z)
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn piaGetByIndex(_env: JNIEnv, _class: JClass, pia: jlong, index: jint) -> jint {
    let storage = PIA_STORAGE.read();
    let pia = storage.get(pia as usize).unwrap();
    pia.get_by_index(index)
}
