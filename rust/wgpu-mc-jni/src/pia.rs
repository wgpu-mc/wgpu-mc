use std::slice;

use jni::objects::{JClass, ReleaseMode};
use jni::sys::{jint, jlong, jlongArray};
use jni::JNIEnv;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use slab::Slab;

pub static PIA_STORAGE: OnceCell<RwLock<Slab<PackedIntegerArray>>> = OnceCell::new();

#[derive(Debug)]
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

    pub fn debug_pointer(&self, index: i32) -> usize {
        assert!(index < self.size, "index: {}, size: {}", index, self.size);

        let i: i32 = self.compute_storage_index(index);

        unsafe { self.data.as_ptr().offset(i as isize) as usize }
    }

    pub fn get_by_index(&self, index: i32) -> i32 {
        assert!(index < self.size, "index: {}, size: {}", index, self.size);

        let i: i32 = self.compute_storage_index(index);

        let ptr = unsafe { self.data.as_ptr().offset(i as isize) };

        let l: i64 = unsafe { ptr.read_volatile() };
        // let l: i64 = i64::from_be_bytes(self.data[i as usize].to_ne_bytes());
        // let l: i64 = self.data[i as usize];

        // (index - i * this.elementsPerLong) * this.elementBits
        let j: i32 = (index - (i * self.elements_per_long)) * self.element_bits;
        ((l >> j) & self.max_value) as i32
    }

    pub fn compute_storage_index(&self, index: i32) -> i32 {
        let l = self.index_scale as u32 as i64;
        let m = self.index_offset as u32 as i64;

        // println!("l {} m {} idxs {}", l, m, self.index_shift);

        (((((index as i64) * l) + m) >> 32) >> self.index_shift) as i32
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createPaletteStorage(
    env: JNIEnv,
    _class: JClass,
    data: jlongArray,
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

    let copy = env
        .get_primitive_array_critical(data, ReleaseMode::NoCopyBack)
        .unwrap();

    let packed_arr = PackedIntegerArray {
        data: Vec::from(unsafe {
            slice::from_raw_parts(copy.as_ptr() as *mut jlong, copy.size().unwrap() as usize)
        })
        .into_boxed_slice(),
        elements_per_long,
        element_bits,
        max_value,
        index_scale,
        index_offset,
        index_shift,
        size,
    };

    let mut storage = PIA_STORAGE.get().unwrap().write();
    storage.insert(packed_arr) as jlong
}

#[allow(unused_must_use)]
#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_destroyPaletteStorage(
    _env: JNIEnv,
    _class: JClass,
    storage: jlong,
) {
    let mut storage_access = PIA_STORAGE.get().unwrap().write();
    storage_access.remove(storage as usize);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_piaGet(
    _env: JNIEnv,
    _class: JClass,
    pia: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jint {
    let storage = PIA_STORAGE.get().unwrap().read();
    let pia = storage.get(pia as usize).unwrap();
    pia.get(x, y, z)
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_piaGetByIndex(
    _env: JNIEnv,
    _class: JClass,
    pia: jlong,
    index: jint,
) -> jint {
    let storage = PIA_STORAGE.get().unwrap().read();
    let pia = storage.get(pia as usize).unwrap();
    pia.get_by_index(index)
}
