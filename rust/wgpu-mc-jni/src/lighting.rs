use std::fmt::Debug;

use jni::JNIEnv;
use jni::objects::JClass;
use jni_fn::jni_fn;
use once_cell::sync::Lazy;

use wgpu_mc::mc::chunk::SECTIONS_PER_CHUNK;

pub static LIGHTMAP_GLID: Lazy<std::sync::Mutex<u32>> = Lazy::new(|| std::sync::Mutex::new(0));

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DeserializedLightData {
    pub sky_light: [u8; 2048 * SECTIONS_PER_CHUNK],
    pub block_light: [u8; 2048 * SECTIONS_PER_CHUNK],
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setLightmapID(_env: JNIEnv, _class: JClass, gl_id: u32) {
    *LIGHTMAP_GLID.try_lock().unwrap() = gl_id;
}
