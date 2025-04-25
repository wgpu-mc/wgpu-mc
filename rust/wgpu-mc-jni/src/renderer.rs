use byteorder::LittleEndian;
use jni::objects::{AutoElements, JClass, JFloatArray, ReleaseMode};
use jni::sys::{jfloat, jint, jlong};
use jni::{objects::JString, JNIEnv};
use jni_fn::jni_fn;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::slice;
use std::{sync::Arc, time::Instant};
use wgpu_mc::mc::entity::{BundledEntityInstances, InstanceVertex};
use wgpu_mc::mc::RenderEffectsData;
use wgpu_mc::texture::BindableTexture;

use crate::application::{load_shaders, SHOULD_STOP};
use crate::RENDERER;

pub static MATRICES: Lazy<Mutex<Matrices>> = Lazy::new(|| {
    Mutex::new(Matrices {
        projection: [[0.0; 4]; 4],
        view: [[0.0; 4]; 4],
        terrain_transformation: [[0.0; 4]; 4],
    })
});

pub struct Matrices {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub terrain_transformation: [[f32; 4]; 4],
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn reloadShaders(_env: JNIEnv, _class: JClass) {
    load_shaders(RENDERER.get().unwrap());
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setMatrix(mut env: JNIEnv, _class: JClass, id: jint, float_array: JFloatArray) {
    let elements: AutoElements<jfloat> =
        unsafe { env.get_array_elements(&float_array, ReleaseMode::NoCopyBack) }.unwrap();

    let slice = unsafe { slice::from_raw_parts(elements.as_ptr(), elements.len()) };

    let mut cursor = Cursor::new(bytemuck::cast_slice::<f32, u8>(slice));
    let mut converted = Vec::with_capacity(slice.len());

    for _ in 0..slice.len() {
        use byteorder::ReadBytesExt;
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    let slice_4x4: [[f32; 4]; 4] = *bytemuck::from_bytes(bytemuck::cast_slice(&converted));

    match id {
        0 => {
            MATRICES.lock().projection = slice_4x4;
        }
        1 => {
            // MATRICES.lock(). = slice_4x4;
        }
        2 => {
            MATRICES.lock().view = slice_4x4;
        }
        3 => {
            MATRICES.lock().terrain_transformation = slice_4x4;
        }
        _ => {}
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn scheduleStop(_env: JNIEnv, _class: JClass) {
    let _ = SHOULD_STOP.set(());
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum MCTextureId {
    BlockAtlas,
    Lightmap,
}

pub static ENTITY_INSTANCES: Lazy<Mutex<HashMap<String, BundledEntityInstances>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static MC_TEXTURES: Lazy<Mutex<HashMap<MCTextureId, Arc<BindableTexture>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn clearEntities(_env: JNIEnv, _class: JClass) {
    ENTITY_INSTANCES.lock().clear();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setEntityInstanceBuffer(
    mut env: JNIEnv,
    _class: JClass,
    entity_name: JString,
    mat4_ptr: jlong,
    mat4_len: jint,
    overlay_ptr: jlong,
    overlay_len: jint,
    instance_count: jint,
    texture_id: jint,
) -> jlong {
    assert!(instance_count >= 0);
    let now = Instant::now();
    let instance_count = instance_count as u32;

    let wm = RENDERER.get().unwrap();

    //TODO this is slow, let's use an integer id somewhere
    let entity_name: String = env.get_string(&entity_name).unwrap().into();

    if instance_count == 0 {
        ENTITY_INSTANCES.lock().remove(&entity_name);
        return Instant::now().duration_since(now).as_nanos() as jlong;
    }

    let mat4s = unsafe { slice::from_raw_parts(mat4_ptr as usize as *mut f32, mat4_len as usize) };

    let overlays =
        unsafe { slice::from_raw_parts(overlay_ptr as usize as *mut i32, overlay_len as usize) };

    let transforms: Vec<f32> = Vec::from(mat4s);

    let verts: Vec<InstanceVertex> = overlays
        .iter()
        .map(|overlay| InstanceVertex {
            uv_offset: [0, 0],
            overlay: *overlay as u32,
        })
        .collect();

    let mut instances = ENTITY_INSTANCES.lock();

    // let to_upload = match instances.get_mut(&entity_name) {
    //     Some(bundled_entity_instances) if bundled_entity_instances.capacity <= instance_count => {
    //         bundled_entity_instances.capacity = instance_count;
    //
    //         bundled_entity_instances
    //     }
    //     _ => {
    //
    //         // TODO
    //         // let texture = todo!();
    //         // let models = wm.mc.entity_models.read();
    //         // let entity = models.get(&entity_name).unwrap();
    //         // instances.insert(
    //         //     entity_name.clone(),
    //         //     BundledEntityInstances::new(wm, entity.clone(), &texture.tv.view, 4096),
    //         // );
    //         // instances.get(&entity_name).unwrap()
    //     }
    // };
    //
    // wm.display.queue.write_buffer(
    //     &to_upload.uploaded.instance_vbo,
    //     0,
    //     bytemuck::cast_slice(&verts),
    // );
    // wm.display.queue.write_buffer(
    //     &to_upload.uploaded.transforms_buffer,
    //     0,
    //     bytemuck::cast_slice(&transforms),
    // );

    Instant::now().duration_since(now).as_nanos() as jlong
}
