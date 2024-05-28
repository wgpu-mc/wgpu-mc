use std::collections::HashMap;
use std::io::Cursor;
use std::{slice, thread};
use std::{sync::Arc, time::Instant};

use byteorder::LittleEndian;
use jni::objects::{AutoElements, JClass, JFloatArray, ReleaseMode};
use jni::sys::{jfloat, jint, jlong};
use jni::{
    objects::{JString, JValue},
    JNIEnv,
};
use jni_fn::jni_fn;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::{Mutex, RwLock};
use wgpu_mc::mc::{RenderEffectsData, SkyState};
use wgpu_mc::mc::entity::{BundledEntityInstances, InstanceVertex, UploadedEntityInstances};
use wgpu_mc::texture::{BindableTexture, TextureAndView};

use crate::application::SHOULD_STOP;
use crate::gl::{ GlTexture, GL_ALLOC};
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
pub fn identifyGlTexture(_env: JNIEnv, _class: JClass, texture: jint, gl_id: jint) {
    let alloc_read = GL_ALLOC.read();
    let gl_texture = alloc_read.get(&(gl_id as u32)).unwrap();

    let mut mc_textures = MC_TEXTURES.lock();
    mc_textures.insert(
        match texture {
            0 => MCTextureId::BlockAtlas,
            1 => MCTextureId::Lightmap,
            _ => unreachable!(),
        },
        gl_texture.bindable_texture.as_ref().unwrap().clone(),
    );
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
    let overlays: Vec<i32> = Vec::from(overlays);
    let verts: Vec<InstanceVertex> = (0..instance_count)
        .map(|index| InstanceVertex {
            entity_index: index,
            uv_offset: [0, 0],
        })
        .collect();

    let mut instances = ENTITY_INSTANCES.lock();
    let bundled_entity_instances =
        if let Some(bundled_entity_instances) = instances.get_mut(&entity_name) {
            bundled_entity_instances.count = instance_count;
            bundled_entity_instances
        } else {
            let texture = {
                let gl_alloc = GL_ALLOC.read();

                match gl_alloc.get(&(texture_id as u32)) {
                    None => return 0,
                    Some(GlTexture {
                        bindable_texture: None,
                        ..
                    }) => return 0,
                    _ => {}
                }

                gl_alloc
                    .get(&(texture_id as u32))
                    .unwrap()
                    .bindable_texture
                    .as_ref()
                    .unwrap()
                    .clone()
            };
            let models = wm.mc.entity_models.read();
            let entity = models.get(&entity_name).unwrap();
            instances.insert(
                entity_name.clone(),
                BundledEntityInstances::new(wm, entity.clone(), instance_count, texture),
            );
            instances.get(&entity_name).unwrap()
        };

    wm.display.queue.write_buffer(
        bundled_entity_instances.uploaded.instance_vbo.as_ref(),
        0,
        bytemuck::cast_slice(&verts),
    );
    wm.display.queue.write_buffer(
        &bundled_entity_instances.uploaded.transform_ssbo.buffer,
        0,
        bytemuck::cast_slice(&transforms),
    );
    wm.display.queue.write_buffer(
        &bundled_entity_instances.uploaded.overlay_ssbo.buffer,
        0,
        bytemuck::cast_slice(&overlays),
    );
    Instant::now().duration_since(now).as_nanos() as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bindSkyData(
    _env: JNIEnv,
    _class: JClass,
    r: jfloat,
    g: jfloat,
    b: jfloat,
    angle: jfloat,
    brightness: jfloat,
    star_shimmer: jfloat,
    moon_phase: jint,
) {
    // let mut sky_data = (**RENDERER.get().unwrap().mc.sky_data.load()).clone();
    // sky_data.color_r = r;
    // sky_data.color_g = g;
    // sky_data.color_b = b;
    // sky_data.angle = angle;
    // sky_data.brightness = brightness;
    // sky_data.star_shimmer = star_shimmer;
    // sky_data.moon_phase = moon_phase;
    //
    // RENDERER.get().unwrap().mc.sky_data.swap(Arc::new(sky_data));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bindRenderEffectsData(
    env: JNIEnv,
    _class: JClass,
    fog_start: jfloat,
    fog_end: jfloat,
    fog_shape: jint,
    fog_color: JFloatArray,
    color_modulator: JFloatArray,
    dimension_fog_color: JFloatArray,
) {
    let mut render_effects_data = RenderEffectsData {
        fog_start,
        fog_end,
        fog_shape: fog_shape as f32,
        ..Default::default()
    };

    let mut fog_color_vec = vec![0f32; env.get_array_length(&fog_color).unwrap() as usize];
    env.get_float_array_region(&fog_color, 0, &mut fog_color_vec[..])
        .unwrap();

    let mut color_modulator_vec =
        vec![0f32; env.get_array_length(&color_modulator).unwrap() as usize];
    env.get_float_array_region(&color_modulator, 0, &mut color_modulator_vec[..])
        .unwrap();

    let mut dimension_fog_color_vec =
        vec![0f32; env.get_array_length(&dimension_fog_color).unwrap() as usize];
    env.get_float_array_region(&dimension_fog_color, 0, &mut dimension_fog_color_vec[..])
        .unwrap();

    // render_effects_data.fog_color = fog_color_vec;
    // render_effects_data.color_modulator = color_modulator_vec;
    // render_effects_data.dimension_fog_color = dimension_fog_color_vec;
    //
    // RENDERER
    //     .get()
    //     .unwrap()
    //     .mc
    //     .render_effects
    //     .swap(Arc::new(render_effects_data));
}
