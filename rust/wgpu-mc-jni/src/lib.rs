pub extern crate wgpu_mc;

use arc_swap::access::Access;
use arc_swap::{ArcSwap, ArcSwapAny};
use byteorder::{LittleEndian, ReadBytesExt};
use core::slice;
use crossbeam_channel::{unbounded, Receiver, Sender};
use glam::{ivec2, ivec3, IVec3, Mat4};
use jni::objects::{
    AutoElements, GlobalRef, JByteArray, JClass, JFloatArray, JIntArray, JLongArray, JObject,
    JObjectArray, JPrimitiveArray, JString, JValue, JValueOwned, ReleaseMode, WeakRef,
};
use jni::sys::{jboolean, jbyte, jfloat, jint, jlong, jsize, jstring, JNI_FALSE, JNI_TRUE};
use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use once_cell::sync::{Lazy, OnceCell};
use palette::PALETTE_STORAGE;
use parking_lot::{Mutex, RwLock};
use pia::PIA_STORAGE;
use rayon::{ThreadPool, ThreadPoolBuilder};
use renderer::MATRICES;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{stdout, Cursor, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::{mem, thread};
use wgpu::Extent3d;
use wgpu_mc::render::graph::{Geometry, RenderGraph, ResourceBacking};
use wgpu_mc::wgpu::util::DeviceExt;

use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{bake_section, BlockStateProvider, LightLevel};
use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::mc::{RenderEffectsData, Scene, SkyState};
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::texture::{BindableTexture, TextureAndView};
use wgpu_mc::wgpu::ImageDataLayout;
use wgpu_mc::wgpu::{self, TextureFormat};
use wgpu_mc::{Frustum, WmRenderer};

use crate::lighting::DeserializedLightData;
use crate::palette::JavaPalette;
use crate::pia::PackedIntegerArray;
use crate::renderer::ENTITY_INSTANCES;
use crate::settings::Settings;

mod alloc;
mod application;
pub mod entity;
mod gl;
mod lighting;
mod palette;
mod pia;
mod renderer;
mod settings;
mod device;

#[derive(Debug)]
struct MinecraftRenderState {
    //draw_queue: Vec<>,
    _render_world: bool,
}

#[allow(dead_code)]
struct MouseState {
    pub x: f64,
    pub y: f64,
}

// static ENTITIES: OnceCell<HashMap<>> = OnceCell::new();
static RENDERER: OnceCell<WmRenderer> = OnceCell::new();

pub static RENDER_GRAPH: OnceCell<Mutex<RenderGraph>> = OnceCell::new();
pub static CUSTOM_GEOMETRY: OnceCell<Mutex<HashMap<String, Box<dyn Geometry>>>> = OnceCell::new();

static RUN_DIRECTORY: OnceCell<PathBuf> = OnceCell::new();
static JVM: OnceCell<RwLock<JavaVM>> = OnceCell::new();
static YARN_CLASS_LOADER: OnceCell<GlobalRef> = OnceCell::new();

type Task = Box<dyn FnOnce() + Send + Sync>;

static TASK_CHANNELS: Lazy<(Sender<Task>, Receiver<Task>)> = Lazy::new(unbounded);
static MC_STATE: Lazy<ArcSwap<MinecraftRenderState>> = Lazy::new(|| {
    ArcSwap::new(Arc::new(MinecraftRenderState {
        _render_world: false,
    }))
});

static CLEAR_COLOR: Lazy<ArcSwap<[f32; 3]>> = Lazy::new(|| ArcSwap::new(Arc::new([0.0; 3])));

static AIR: Lazy<BlockstateKey> = Lazy::new(|| BlockstateKey {
    block: RENDERER
        .get()
        .unwrap()
        .mc
        .block_manager
        .read()
        .blocks
        .get_full("minecraft:air")
        .unwrap()
        .0 as u16,
    augment: 0,
});

static BLOCKS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static BLOCK_STATES: Mutex<Vec<(String, String, GlobalRef)>> = Mutex::new(Vec::new());
pub static SETTINGS: RwLock<Option<Settings>> = RwLock::new(None);

pub static CLASSLOADER: OnceCell<WeakRef> = OnceCell::new();

pub fn call_static_from_class_loader<'env>(
    env: &mut JNIEnv<'env>,
    class: &str,
    method: &str,
    sig: &str,
    args: &[JValue],
) -> jni::errors::Result<JValueOwned<'env>> {
    let class_loader = CLASSLOADER
        .get()
        .unwrap()
        .upgrade_local(&*env)
        .unwrap()
        .unwrap();
    let arg = env.new_string(class).unwrap();
    let class_obj: JClass = env
        .call_method(
            class_loader,
            "findClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&arg)],
        )
        .unwrap()
        .l()
        .unwrap()
        .into();
    env.call_static_method(class_obj, method, sig, args)
}

#[derive(Debug)]
pub struct SectionHolder {
    pub block_data: Option<(JavaPalette, PackedIntegerArray)>,
    pub light_data: Option<DeserializedLightData>,
}

#[derive(Debug)]
pub struct MinecraftBlockstateProvider {
    pub sections: [Option<SectionHolder>; 27],
    pub air: BlockstateKey,
}
impl BlockStateProvider for MinecraftBlockstateProvider {
    fn get_state(&self, pos: IVec3) -> ChunkBlockState {
        let section_pos: IVec3 = (pos >> 4) + 1;
        let section_option =
            &self.sections[(section_pos.x + section_pos.y * 3 + section_pos.z * 9) as usize];

        let section = match section_option {
            None => return ChunkBlockState::Air,
            Some(chunk) => chunk,
        };

        let (palette, storage) = match &section.block_data {
            Some(section) => section,
            None => return ChunkBlockState::Air,
        };

        let palette_key = storage.get(pos.x & 15, pos.y & 15, pos.z & 15);
        let block = palette.get(palette_key as usize).unwrap();

        if *block == self.air {
            ChunkBlockState::Air
        } else {
            ChunkBlockState::State(*block)
        }
    }

    fn get_light_level(&self, pos: IVec3) -> LightLevel {
        let section_pos: IVec3 = (pos >> 4) + 1;
        let chunk_option =
            &self.sections[(section_pos.x + section_pos.y * 3 + section_pos.z * 9) as usize];

        let chunk = match chunk_option {
            None => return LightLevel::from_sky_and_block(0, 0),
            Some(chunk) => chunk,
        };

        let light_data = match &chunk.light_data {
            None => return LightLevel::from_sky_and_block(0, 0),
            Some(light_data) => light_data,
        };

        let local_x = pos.x & 0b1111;
        let local_y = pos.y & 0b1111;
        let local_z = pos.z & 0b1111;

        let packed_coords = ((local_y << 8) | (local_z << 4) | (local_x)) as usize;

        let shift = (packed_coords & 1) << 2;

        let array_index = packed_coords >> 1;

        let sky_light = (light_data.sky_light[array_index] >> shift) & 0b1111;
        let block_light = (light_data.block_light[array_index] >> shift) & 0b1111;

        LightLevel::from_sky_and_block(sky_light, block_light)
    }

    fn is_section_empty(&self, rel_pos: IVec3) -> bool {
        if rel_pos.abs().cmpgt(ivec3(1, 1, 1)).any() {
            return true;
        }

        self.sections[(rel_pos + 1).dot(ivec3(1, 3, 9)) as usize].is_none()
    }

    fn get_block_color(&self, _pos: IVec3, _tint_index: i32) -> u32 {
        0xffffffff
    }
}

struct MinecraftResourceManagerAdapter {
    jvm: JavaVM,
}

impl ResourceProvider for MinecraftResourceManagerAdapter {
    fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>> {
        let mut env = self.jvm.attach_current_thread().unwrap();

        let path = env.new_string(&id.0).unwrap();

        let bytes: JByteArray = call_static_from_class_loader(
            &mut env,
            "dev.birb.wgpu.rust.WgpuResourceProvider",
            "getResource",
            "(Ljava/lang/String;)[B",
            &[JValue::Object(&path.into())],
        )
        .expect(&id.0)
        .l()
        .expect(&id.0)
        .into();

        let elements: AutoElements<jbyte> =
            unsafe { env.get_array_elements(&bytes, ReleaseMode::NoCopyBack) }.unwrap();

        let size = elements.len();
        // let vec = elements.iter().map(|&x| x as u8).collect::<Vec<_>>();

        Some(Vec::from(unsafe {
            slice::from_raw_parts(elements.as_ptr() as *const u8, size)
        }))
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getSettingsStructure(env: JNIEnv, _class: JClass) -> jstring {
    env.new_string(crate::settings::SETTINGS_INFO_JSON.clone())
        .unwrap()
        .into_raw()
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getSettings(env: JNIEnv, _class: JClass) -> jstring {
    let json = serde_json::to_string(&SETTINGS.read().as_ref().unwrap()).unwrap();
    env.new_string(json).unwrap().into_raw()
}

/// Returns true if succeeded and false if not.
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn sendSettings(mut env: JNIEnv, _class: JClass, settings: JString) -> bool {
    let json: String = env.get_string(&settings).unwrap().into();
    if let Ok(settings) = serde_json::from_str(json.as_str()) {
        let mut guard = SETTINGS.write();
        *guard = Some(settings);
        true
    } else {
        false
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn sendRunDirectory(mut env: JNIEnv, _class: JClass, dir: JString) {
    let dir: String = env.get_string(&dir).unwrap().into();
    let path = PathBuf::from(dir);
    RUN_DIRECTORY.set(path).unwrap();

    let mut write = SETTINGS.write();
    *write = Some(Settings::load_or_default());
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getBackend(env: JNIEnv, _class: JClass) -> jstring {
    let renderer = RENDERER.get().unwrap();
    let backend = renderer.get_backend_description();

    env.new_string(backend).unwrap().into_raw()
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerBlockState(
    mut env: JNIEnv,
    _class: JClass,
    block_state: JObject,
    block_name: JString,
    state_key: JString,
) {
    let global_ref = env.new_global_ref(block_state).unwrap();

    let block_name: String = env.get_string(&block_name).unwrap().into();
    let state_key: String = env.get_string(&state_key).unwrap().into();

    BLOCK_STATES
        .lock()
        .push((block_name, state_key, global_ref));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn reloadStorage(_env: JNIEnv, _class: JClass, clampedViewDistance: jint, scene: jlong) {
    let scene = unsafe { &mut *(scene as *mut Scene) };
    
    let mut section_storage = scene.section_storage.write();
    section_storage.clear();
    section_storage.set_width(clampedViewDistance);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setSectionPos(_env: JNIEnv, _class: JClass, x: jint, z: jint, scene: jlong) {
    let scene = unsafe { &mut *(scene as *mut Scene) };
    *scene.camera_section_pos.write() = ivec2(x, z);
}

struct MinecraftBlockStateProviderWrapper<'a> {
    internal: MinecraftBlockstateProvider,
    env: RefCell<JNIEnv<'a>>,
}

impl<'a> BlockStateProvider for MinecraftBlockStateProviderWrapper<'a> {
    fn get_state(&self, pos: IVec3) -> ChunkBlockState {
        self.internal.get_state(pos)
    }

    fn get_light_level(&self, pos: IVec3) -> LightLevel {
        self.internal.get_light_level(pos)
    }

    fn is_section_empty(&self, rel_pos: IVec3) -> bool {
        self.internal.is_section_empty(rel_pos)
    }

    fn get_block_color(&self, pos: IVec3, tint_index: i32) -> u32 {
        self.env
            .borrow_mut()
            .call_static_method(
                "dev/birb/wgpu/render/Wgpu",
                "helperGetBlockColor",
                "(IIII)I",
                &[
                    JValue::Int(pos.x),
                    JValue::Int(pos.y),
                    JValue::Int(pos.z),
                    JValue::Int(tint_index),
                ],
            )
            .unwrap()
            .i()
            .unwrap() as u32
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bakeSection(
    mut env: JNIEnv,
    _class: JClass,
    x: jint,
    y: jint,
    z: jint,
    paletteIndices: JLongArray,
    storageIndices: JLongArray,
    blockBytes: JObjectArray,
    skyBytes: JObjectArray,
) {
    let palette_elements =
        unsafe { env.get_array_elements(&paletteIndices, ReleaseMode::NoCopyBack) }.unwrap();
    let palettes =
        unsafe { slice::from_raw_parts(palette_elements.as_ptr(), palette_elements.len()) };
    let storage_elements =
        unsafe { env.get_array_elements(&storageIndices, ReleaseMode::NoCopyBack) }.unwrap();
    let storages =
        unsafe { slice::from_raw_parts(storage_elements.as_ptr(), storage_elements.len()) };
    const NONE: Option<SectionHolder> = None;
    let mut bsp = MinecraftBlockstateProvider {
        sections: [NONE; 27],
        air: *AIR,
    };

    for i in 0..27 {
        let mut palette_storage = PALETTE_STORAGE.write();
        let mut pia_storage = PIA_STORAGE.write();

        let block_data = if palette_storage.contains(palettes[i] as usize)
            && pia_storage.contains(storages[i] as usize)
        {
            Some((
                palette_storage.remove(palettes[i] as usize),
                pia_storage.remove(storages[i] as usize),
            ))
        } else {
            None
        };
        let sky_array = unsafe {
            JPrimitiveArray::from_raw(
                env.get_object_array_element(&skyBytes, i as jsize)
                    .unwrap()
                    .into_raw(),
            )
        };
        let sky_bytes =
            unsafe { env.get_array_elements(&sky_array, ReleaseMode::NoCopyBack) }.unwrap();
        let block_array = unsafe {
            JPrimitiveArray::from_raw(
                env.get_object_array_element(&blockBytes, i as jsize)
                    .unwrap()
                    .into_raw(),
            )
        };
        let block_bytes =
            unsafe { env.get_array_elements(&block_array, ReleaseMode::NoCopyBack) }.unwrap();

        bsp.sections[i] = Some(SectionHolder {
            block_data,
            light_data: Some(DeserializedLightData {
                sky_light: Box::new(
                    unsafe { slice::from_raw_parts(sky_bytes.as_ptr(), sky_bytes.len()) }
                        .try_into()
                        .unwrap(),
                ),
                block_light: Box::new(
                    unsafe { slice::from_raw_parts(block_bytes.as_ptr(), block_bytes.len()) }
                        .try_into()
                        .unwrap(),
                ),
            }),
        });
    }

    // THREAD_POOL.get().unwrap().spawn(move || {
    let wm = RENDERER.get().unwrap();
    // let env = jvm.attach_current_thread_as_daemon().unwrap();

    let wrapper = MinecraftBlockStateProviderWrapper {
        internal: bsp,
        env: RefCell::new(env),
    };

    bake_section(ivec3(x, y, z), wm, &wrapper);
    // })
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerBlock(mut env: JNIEnv, _class: JClass, name: JString) {
    let name: String = env.get_string(&name).unwrap().into();

    BLOCKS.lock().push(name);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn render(_env: JNIEnv, _class: JClass, _tick_delta: jfloat, _start_time: jlong, _tick: jlong, scene: jlong, width: jint, height: jint) {
    let width = width as u32;
    let height = height as u32;

    let wm = RENDERER.get().unwrap();
    let render_graph = RENDER_GRAPH.get().unwrap().lock();
    let mut geometry = CUSTOM_GEOMETRY.get().unwrap().lock();
    let scene = unsafe { &mut *(scene as *mut Scene) };

    wm.submit_chunk_updates(scene);
    let pos = *scene.camera_section_pos.read();
    scene.section_storage.write().trim(pos);
    *scene.entity_instances.lock() = ENTITY_INSTANCES.lock().clone();

    let matrices = MATRICES.lock();
    if let ResourceBacking::Buffer(buffer, _) = &render_graph.resources["@mat4_perspective"] {
        wm.gpu
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&matrices.projection));
    }
    if let ResourceBacking::Buffer(buffer, _) = &render_graph.resources["@mat4_view"] {
        wm.gpu
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&matrices.view));
    }
    if let ResourceBacking::Buffer(buffer, _) = &render_graph.resources["@mat4_model"] {
        wm.gpu.queue.write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(&matrices.terrain_transformation),
        );
    }

    let texture = wm
        .gpu
        .surface
        .get_current_texture()
        .unwrap_or_else(|_| {
            //The surface is outdated, so we force an update. This can't be done on the window resize event for synchronization reasons.

            let mut surface_config = wm.gpu.config.write();
            surface_config.width = width;
            surface_config.height = height;
            scene.resize_depth_texture(wm, width, height);
            wm.gpu
                .surface
                .configure(&wm.gpu.device, &surface_config);
            wm.gpu.surface.get_current_texture().unwrap()
        });

    let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: Some(TextureFormat::Bgra8Unorm),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: Default::default(),
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    {
        let mut encoder = wm
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let clear_color =
            *<Lazy<ArcSwapAny<Arc<[f32; 3]>>> as Access<[f32; 3]>>::load(&CLEAR_COLOR);
        render_graph.render(
            wm,
            &mut encoder,
            scene,
            &view,
            clear_color,
            &mut geometry,
            &Frustum::from_modelview_projection([[0.0; 4]; 4]),
        );

        wm.gpu.queue.submit([encoder.finish()]);
    }

    texture.present();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn cacheBlockStates(mut env: JNIEnv, _class: JClass) {
    let wm = RENDERER.get().unwrap();
    {
        let blocks = BLOCKS.lock();

        let blockstates = blocks
            .iter()
            .map(|identifier| {
                (
                    identifier.clone(),
                    ResourcePath::from(&identifier[..])
                        .prepend("blockstates/")
                        .append(".json"),
                )
            })
            .collect::<Vec<_>>();

        wm.mc.bake_blocks(
            wm,
            blockstates
                .iter()
                .map(|(string, resource)| (string, resource)),
        );
    }

    let mut states = BLOCK_STATES.lock();

    let block_manager = wm.mc.block_manager.write();
    let mut mappings = Vec::new();

    let mut stdout = stdout().lock();

    states
        .iter()
        .for_each(|(block_name, state_key, global_ref)| {
            let (id_key, _, wm_block) = block_manager.blocks.get_full(block_name).unwrap();

            let key_iter = if !state_key.is_empty() {
                state_key
                    .split(',')
                    .filter_map(|kv_pair| {
                        let mut split = kv_pair.split('=');
                        if kv_pair.is_empty() {
                            return None;
                        }

                        Some((
                            split.next().unwrap(),
                            match split.next().unwrap() {
                                "true" => StateValue::Bool(true),
                                "false" => StateValue::Bool(false),
                                other => StateValue::String(other.into()),
                            },
                        ))
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            let atlases = wm.mc.texture_manager.atlases.write();
            let atlas = &atlases[BLOCK_ATLAS];
            let model = wm_block.get_model_by_key(
                key_iter
                    .iter()
                    .filter(|(a, _)| *a != "waterlogged")
                    .map(|(a, b)| (*a, b)),
                &*wm.mc.resource_provider,
                atlas,
                0,
            );
            let fallback_key = block_manager.blocks.get_full("minecraft:bedrock").unwrap();

            let key = match model {
                Some((_, augment)) => BlockstateKey {
                    block: id_key as u16,
                    augment,
                },
                None => BlockstateKey {
                    block: fallback_key.0 as u16,
                    augment: 0,
                },
            };

            if key.block == fallback_key.0 as u16 {
                writeln!(&mut stdout, "{} {}", block_name, state_key).unwrap();
            }

            mappings.push((key, global_ref));
        });

    drop(stdout);

    mappings.iter().for_each(|(blockstate_key, global_ref)| {
        env.call_static_method(
            "dev/birb/wgpu/render/Wgpu",
            "helperSetBlockStateIndex",
            "(Ljava/lang/Object;I)V",
            &[
                JValue::Object(global_ref.as_obj()),
                JValue::Int(blockstate_key.pack() as i32),
            ],
        )
        .unwrap();
    });

    let instant = Instant::now();

    let state_count = states.len();

    states.clear();

    let debug_message = format!(
        "Released {} global refs to BlockState objects in {}ms",
        state_count,
        Instant::now().duration_since(instant).as_millis()
    );

    let debug_jstring = env.new_string(debug_message).unwrap();

    env.call_static_method(
        "dev/birb/wgpu/render/Wgpu",
        "rustDebug",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&unsafe {
            JObject::from_raw(debug_jstring.into_raw())
        })],
    )
    .unwrap();
}

#[allow(unused_must_use)]
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setPanicHook(env: JNIEnv, _class: JClass) {
    env_logger::init();

    let jvm = env.get_java_vm().unwrap();
    let jvm_ptr = jvm.get_java_vm_pointer() as usize;

    std::panic::set_hook(Box::new(move |panic_info| {
        println!("{panic_info}");

        let jvm = unsafe { JavaVM::from_raw(jvm_ptr as _).unwrap() };
        let mut env = jvm.attach_current_thread_permanently().unwrap();

        let message = format!("wgpu-mc has panicked. Minecraft will now exit.\n{panic_info}");
        let jstring = env.new_string(message).unwrap();

        //Does not return
        env.call_static_method(
            "dev/birb/wgpu/render/Wgpu",
            "rustPanic",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&JObject::from(jstring))],
        );
    }))
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setWorldRenderState(_env: JNIEnv, _class: JClass, boolean: jboolean) {
    MC_STATE.store(Arc::new(MinecraftRenderState {
        _render_world: boolean != 0,
    }));
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
    scene: jlong
) {
    let scene = unsafe { &mut *(scene as *mut Scene) };

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

    render_effects_data.fog_color = [
        fog_color_vec[0],
        fog_color_vec[1],
        fog_color_vec[2],
        fog_color_vec[3],
    ];
    render_effects_data.color_modulator = [
        color_modulator_vec[0],
        color_modulator_vec[1],
        color_modulator_vec[2],
        color_modulator_vec[3],
    ];
    render_effects_data.dimension_fog_color = [
        dimension_fog_color_vec[0],
        dimension_fog_color_vec[1],
        dimension_fog_color_vec[2],
        dimension_fog_color_vec[3],
    ];

    CLEAR_COLOR.swap([fog_color_vec[0], fog_color_vec[1], fog_color_vec[2]].into());

    scene.render_effects.swap(render_effects_data.into());
}
