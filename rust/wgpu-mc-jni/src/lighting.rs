use std::fmt::Debug;

use jni::objects::JClass;
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::Lazy;

pub static LIGHTMAP_GLID: Lazy<std::sync::Mutex<u32>> = Lazy::new(|| std::sync::Mutex::new(0));

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeserializedLightData {
    pub sky_light: Box<[u8; 2048]>,
    pub block_light: Box<[u8; 2048]>,
}
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setLightmapID(_env: JNIEnv, _class: JClass, gl_id: u32) {
    *LIGHTMAP_GLID.try_lock().unwrap() = gl_id;
}
