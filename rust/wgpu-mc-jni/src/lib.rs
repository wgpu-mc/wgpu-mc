pub extern crate wgpu_mc;

use core::slice;
use std::collections::HashMap;
use std::{mem, ptr, thread};
use std::fmt::Debug;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use application::Application;
use arc_swap::{ArcSwap, AsRaw};
use byteorder::{LittleEndian, ReadBytesExt};
use crossbeam_channel::{Receiver, Sender, unbounded};
use glam::{ivec2, ivec3, IVec3, Mat4};
use jni::{JavaVM, JNIEnv};
use jni::objects::{
    AutoElements, GlobalRef, JByteArray, JClass, JFloatArray, JIntArray, JLongArray, JObject, JObjectArray, JPrimitiveArray, JString, JValue, JValueGen, JValueOwned, ReleaseMode, WeakRef
};
use jni::sys::{
    jboolean, jbyte, jbyteArray, jfloat, jint, jlong, jsize, jstring, JNI_FALSE, JNI_TRUE
};
use jni_fn::jni_fn;
use once_cell::sync::{Lazy, OnceCell};
use palette::PALETTE_STORAGE;
use parking_lot::{Mutex, RwLock};
use pia::PIA_STORAGE;
use rayon::{ThreadPool, ThreadPoolBuilder};
use renderer::MATRICES;
use wgpu::Extent3d;
use wgpu_mc::render::graph::{Geometry, RenderGraph, ResourceBacking};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::window::CursorGrabMode;

use wgpu_mc::{Frustum, WmRenderer};
use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{
    bake_section, BlockStateProvider, LightLevel, Section, CHUNK_HEIGHT
};
use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::mc::Scene;
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::texture::{BindableTexture, TextureAndView};
use wgpu_mc::wgpu::{self, TextureFormat};
use wgpu_mc::wgpu::ImageDataLayout;

use crate::gl::{GL_ALLOC, GL_COMMANDS, GLCommand, GlTexture};
use crate::lighting::DeserializedLightData;
use crate::palette::JavaPalette;
use crate::pia::PackedIntegerArray;
use crate::settings::Settings;

mod alloc;
pub mod entity;
mod gl;
mod lighting;
mod palette;
mod pia;
mod renderer;
mod settings;
mod application;

#[allow(dead_code)]
enum RenderMessage {
    SetTitle(String),
    KeyPressed(u32),
    MouseState(ElementState, MouseButton),
    KeyState(u32, u32, u32, u32),
    CharTyped(char, u32),
    MouseMove(f64, f64),
    CursorMove(f64, f64),
    Resized(u32, u32),
    Focused(bool),
}

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

pub static RENDER_GRAPH: OnceCell<RenderGraph> = OnceCell::new();
pub static CUSTOM_GEOMETRY: OnceCell<Mutex<HashMap<String, Box<dyn Geometry>>>> = OnceCell::new();

static RUN_DIRECTORY: OnceCell<PathBuf> = OnceCell::new();

type Task = Box<dyn FnOnce() + Send + Sync>;

static CHANNELS: Lazy<(Sender<RenderMessage>, Receiver<RenderMessage>)> = Lazy::new(unbounded);
static TASK_CHANNELS: Lazy<(Sender<Task>, Receiver<Task>)> = Lazy::new(unbounded);
static MC_STATE: Lazy<ArcSwap<MinecraftRenderState>> = Lazy::new(|| {
    ArcSwap::new(Arc::new(MinecraftRenderState {
        _render_world: false,
    }))
});

static CLEAR_COLOR: Lazy<ArcSwap<[f32; 3]>> = Lazy::new(|| ArcSwap::new(Arc::new([0.0; 3])));

static THREAD_POOL: Lazy<ThreadPool> =
    Lazy::new(|| ThreadPoolBuilder::new().build().unwrap());

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

static SCENE: Lazy<Scene> = Lazy::new(|| {
    let wm = RENDERER.get().unwrap();

    Scene::new(
        wm,
        wgpu::Extent3d {
            width: wm.display.window.inner_size().width,
            height: wm.display.window.inner_size().height,
            depth_or_array_layers: 1,
        },
    )
});

static BLOCKS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static BLOCK_STATES: Mutex<Vec<(String, String, GlobalRef)>> = Mutex::new(Vec::new());
pub static SETTINGS: RwLock<Option<Settings>> = RwLock::new(None);


pub static CLASSLOADER: OnceCell<WeakRef> = OnceCell::new();

pub fn call_static_from_class_loader<'env>(env:&mut JNIEnv<'env>,class:&str,method:&str,sig:&str,args:&[JValue])->jni::errors::Result<JValueOwned<'env>>{
    let class_loader = CLASSLOADER.get().unwrap().upgrade_local(&*env).unwrap().unwrap();
    let arg = env.new_string(class).unwrap();
    let class_obj:JClass = env.call_method(class_loader,"findClass","(Ljava/lang/String;)Ljava/lang/Class;",&[JValue::Object(&arg)])
        .unwrap().l().unwrap().into();
    env.call_static_method(class_obj,method,sig,args)
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
    fn get_state(&self, x: i32, y: i32, z: i32) -> ChunkBlockState {
        let section_pos = ivec3((x >> 4) + 1, (y >> 4) + 1, (z >> 4) + 1);
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

        let palette_key = storage.get(x & 15, y & 15, z & 15);
        let block = palette.get(palette_key as usize).unwrap();

        if *block == self.air {
            ChunkBlockState::Air
        } else {
            ChunkBlockState::State(*block)
        }
    }

    fn get_light_level(&self, x: i32, y: i32, z: i32) -> LightLevel {
        let section_pos = ivec3((x >> 4) + 1, (y >> 4) + 1, (z >> 4) + 1);
        let chunk_option =
            &self.sections[(section_pos.x + section_pos.y * 3 + section_pos.z * 9) as usize];

        let chunk = match chunk_option {
            None => return LightLevel::from_sky_and_block(0, 0),
            Some(chunk) => chunk,
        };

        if y as usize >= CHUNK_HEIGHT {
            return LightLevel::from_sky_and_block(15, 0);
        } else if y < 0 {
            return LightLevel::from_sky_and_block(0, 0);
        }

        let light_data = match &chunk.light_data {
            None => return LightLevel::from_sky_and_block(0, 0),
            Some(light_data) => light_data,
        };

        let local_x = x & 0b1111;
        let local_y = y & 0b1111;
        let local_z = z & 0b1111;

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

        self.sections[(rel_pos+1).dot(ivec3(1, 3, 9)) as usize].is_none()
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
            "getResource", "(Ljava/lang/String;)[B",
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
pub fn reload(_env: JNIEnv, _class: JClass,clampedViewDistance:jint) {
    let mut section_storage = SCENE.section_storage.write();
    section_storage.clear();
    section_storage.set_width(clampedViewDistance);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setSectionPos(_env: JNIEnv, _class: JClass, x:jint,z:jint){
    *SCENE.camera_section_pos.write() = ivec2(x, z);
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

        let block_data = 
        if palette_storage.contains(palettes[i] as usize) && pia_storage.contains(storages[i] as usize) {
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

    THREAD_POOL.spawn(move || {
        let wm = RENDERER.get().unwrap();
        bake_section(ivec3(x, y, z), wm ,&bsp);
    })
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerBlock(mut env: JNIEnv, _class: JClass, name: JString) {
    let name: String = env.get_string(&name).unwrap().into();

    BLOCKS.lock().push(name);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn startRendering(mut env: JNIEnv, _class: JClass, title: JString) {

    let title: String = env.get_string(&title).unwrap().into();
    let jvm = env.get_java_vm().unwrap();

    let class_loader = env.call_static_method(
        "dev/birb/wgpu/rust/WgpuNative",
        "getClassLoader",
        "()Ljava/lang/ClassLoader;",
        &[],
    )
    .unwrap();
    let Ok(_) = CLASSLOADER.set(env.new_weak_ref::<JObject>(class_loader.try_into().unwrap()).unwrap().unwrap()) else { panic!("Failed to set classloader") };

    let mut application = Application::new(jvm,title);
    let mut event_loop = winit::event_loop::EventLoop::builder();

    #[cfg(target_os = "linux")]
    {
        // double hacky fix B)
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            event_loop.with_any_thread(true);
        } else {
            use winit::platform::x11::EventLoopBuilderExtX11;
            event_loop.with_any_thread(true);
        }
    }
    let event_loop = event_loop.build().unwrap();
    event_loop.run_app(&mut application).unwrap();

}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn render(mut env: JNIEnv, _class: JClass, tick_delta:jfloat, start_time:jlong, tick:jlong) {
    let wm = RENDERER.wait();
    let render_graph = RENDER_GRAPH.get().unwrap();
    let mut geometry = CUSTOM_GEOMETRY.get().unwrap().lock();
    wm.display.window.request_redraw();
    wm.submit_chunk_updates(&SCENE);
    let pos = SCENE.camera_section_pos.read().clone();
    SCENE.section_storage.write().trim(pos);

    let matrices = MATRICES.lock();
    if let ResourceBacking::Buffer(buffer,_) = &render_graph.resources["@mat4_perspective"]{
        wm.display.queue.write_buffer(
            &buffer,
            0,
            bytemuck::cast_slice(&matrices.projection),
        );
    }
    if let ResourceBacking::Buffer(buffer,_) = &render_graph.resources["@mat4_view"]{
        wm.display.queue.write_buffer(
            &buffer,
            0,
            bytemuck::cast_slice(&matrices.view),
        );
    }
    if let ResourceBacking::Buffer(buffer,_) = &render_graph.resources["@mat4_model"]{
        wm.display.queue.write_buffer(
            &buffer,
            0,
            bytemuck::cast_slice(&matrices.terrain_transformation),
        );
    }

    let texture = wm.display.surface.get_current_texture().unwrap_or_else(|_| {
        //The surface is outdated, so we force an update. This can't be done on the window resize event for synchronization reasons.
        
        let mut surface_config = wm.display.config.write();
        let size = wm.display.size.read();
        surface_config.width = size.width;
        surface_config.height = size.height;

        wm.display.surface.configure(&wm.display.device, &surface_config);
        wm.display.surface.get_current_texture().unwrap()
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
        let mut encoder = wm.display.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        render_graph.render(
            &wm,
            &mut encoder,
            &SCENE,
            &view,
            [0; 3],
            &mut geometry,
            &Frustum::from_modelview_projection([[0.0; 4]; 4])
        );

        wm.display.queue.submit([encoder.finish()]);
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
            let atlases = wm
                            .mc
                            .texture_manager
                            .atlases
                            .write();
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

            mappings.push((key, global_ref));
        });

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

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn runHelperThread(mut env: JNIEnv, _class: JClass) {
    //Wait until wgpu-mc is initialized
    while RENDERER.get().is_none() {}
    let wm = RENDERER.get().unwrap();
    thread::spawn(|| {
        let rx = &TASK_CHANNELS.1;
        for task in rx.iter() {
            task()
        }
    });

    let rx = &CHANNELS.1;

    for render_message in rx.iter() {
        match render_message {
            RenderMessage::SetTitle(title) => wm.display.window.set_title(&title),
            RenderMessage::KeyPressed(_) => {}
            RenderMessage::MouseMove(x, y) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "mouseMove",
                    "(DD)V",
                    &[JValue::Double(x), JValue::Double(y)],
                )
                .unwrap();
            }
            RenderMessage::CursorMove(x, y) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "cursorMove",
                    "(DD)V",
                    &[JValue::Double(x), JValue::Double(y)],
                )
                .unwrap();
            }
            RenderMessage::MouseState(element_state, mouse_button) => {
                let button = match mouse_button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    _ => 0,
                };

                let action = match element_state {
                    ElementState::Pressed => 1,
                    ElementState::Released => 0,
                };

                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "mouseAction",
                    "(II)V",
                    &[JValue::Int(button), JValue::Int(action)],
                )
                .unwrap();
            }
            RenderMessage::Resized(width, height) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "onResize",
                    "(II)V",
                    &[JValue::Int(width as i32), JValue::Int(height as i32)],
                )
                .unwrap();
            }
            RenderMessage::KeyState(key, scancode, action, modifiers) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "keyState",
                    "(IIII)V",
                    &[
                        JValue::Int(key as i32),
                        JValue::Int(scancode as i32),
                        JValue::Int(action as i32),
                        JValue::Int(modifiers as i32),
                    ],
                )
                .unwrap();
            }
            RenderMessage::CharTyped(ch, modifiers) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "onChar",
                    "(II)V",
                    &[JValue::Int(ch as i32), JValue::Int(modifiers as i32)],
                )
                .unwrap();
            }
            RenderMessage::Focused(focused) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "windowFocused",
                    "(Z)V",
                    &[JValue::Bool(if focused { JNI_TRUE } else { JNI_FALSE })],
                )
                .unwrap();
            }
        };
    }
}

#[allow(unused_must_use)]
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn centerCursor(_env: JNIEnv, _class: JClass, _locked: jboolean) {
    if let Some(wm) = RENDERER.get() {
        let window = &wm.display.window;
        let inner = window.inner_position().unwrap();
        let size = window.inner_size();
        window
            .set_cursor_position(PhysicalPosition::new(
                inner.x + (size.width as i32 / 2),
                inner.y + (size.height as i32 / 2),
            ))
            .unwrap();
    }
}

#[allow(unused_must_use)]
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setCursorLocked(_env: JNIEnv, _class: JClass, locked: jboolean) {
    if let Some(wm) = RENDERER.get() {
        let window = &wm.display.window;
        if locked == JNI_TRUE {
            window.set_cursor_visible(false);
            window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                .unwrap();
        } else {
            window.set_cursor_visible(true);
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
        }
    }
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

#[allow(unused_must_use)]
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn updateWindowTitle(mut env: JNIEnv, _class: JClass, jtitle: JString) {
    let tx = &CHANNELS.0;

    let title: String = env.get_string(&jtitle).unwrap().into();

    tx.send(RenderMessage::SetTitle(title));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setWorldRenderState(_env: JNIEnv, _class: JClass, boolean: jboolean) {
    MC_STATE.store(Arc::new(MinecraftRenderState {
        _render_world: boolean != 0,
    }));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn submitCommands(_env: JNIEnv, _class: JClass) {
    let mut guard = GL_COMMANDS.write();
    let (command_stack, submitted) = &mut *guard;

    mem::swap(command_stack, submitted);

    command_stack.clear();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn texImage2D(
    _env: JNIEnv,
    _class: JClass,
    texture_id: jint,
    _target: jint,
    _level: jint,
    _internal_format: jint,
    width: jint,
    height: jint,
    _border: jint,
    format: jint,
    _type: jint,
    pixels_ptr: jlong,
) {
    let _pixel_size = match format {
        0x1908 | 0x80E1 => 4,
        _ => panic!("Unknown format {format:x}"),
    };

    //For when the renderer is initialized
    let task = move || {
        let area = width * height;
        //In bytes
        assert_eq!(_type, 0x1401);
        let size = area as usize * 4;

        let data = if pixels_ptr != 0 {
            Vec::from(unsafe { slice::from_raw_parts(pixels_ptr as *const u8, size) })
        } else {
            vec![0; size]
        };

        let wm = RENDERER.get().unwrap();

        let tsv = TextureAndView::from_rgb_bytes(
            &wm.display,
            &data[..],
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            None,
            match format {
                0x1908 => wgpu::TextureFormat::Rgba8Unorm,
                0x80E1 => wgpu::TextureFormat::Bgra8Unorm,
                _ => unimplemented!(),
            },
        )
        .unwrap();

        let bindable = BindableTexture::from_tv(&wm, Arc::new(tsv), false);

        {
            GL_ALLOC.write().insert(
                texture_id as u32,
                GlTexture {
                    width: width as u16,
                    height: height as u16,
                    bindable_texture: Some(Arc::new(bindable)),
                    pixels: data,
                },
            );
        }
    };

    let tx = &TASK_CHANNELS.0;

    tx.send(Box::new(task)).unwrap();
}

#[allow(non_snake_case)]
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn subImage2D(
    mut env: JNIEnv,
    _class: JClass,
    texture_id: jint,
    _target: jint,
    _level: jint,
    offsetX: jint,
    offsetY: jint,
    width: jint,
    height: jint,
    format: jint,
    _type: jint,
    pixels: JIntArray,
    unpack_row_length: jint,
    unpack_skip_pixels: jint,
    unpack_skip_rows: jint,
    unpack_alignment: jint,
) {
    let pixel_array_pointer: AutoElements<jint> =
        unsafe { env.get_array_elements(&pixels, ReleaseMode::NoCopyBack) }.unwrap();
    let pixels = unsafe {
        Vec::from(slice::from_raw_parts(
            pixel_array_pointer.as_ptr() as *mut u32,
            pixel_array_pointer.len(),
        ))
    };
    let unpack_row_length = unpack_row_length as usize;
    let _unpack_skip_pixels = unpack_skip_pixels as usize;
    let _unpack_skip_rows = unpack_skip_rows as usize;
    let _unpack_alignment = unpack_alignment as usize;
    let width = width as usize;
    let height = height as usize;

    let pixel_size = match format {
        0x1908 | 0x80E1 => 4,
        _ => panic!("Unknown format {format:x}"),
    };

    //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glPixelStore.xhtml
    let _row_width = if unpack_row_length > 0 {
        unpack_row_length
    } else {
        width
    };

    //In bytes
    assert_eq!(_type, 0x1401);

    //For when the renderer is initialized
    let task = move || {
        let wm = RENDERER.get().unwrap();

        let mut alloc_write = GL_ALLOC.write();

        let gl_texture = alloc_write.get_mut(&(texture_id as u32)).unwrap();

        let dest_row_size = gl_texture.width as usize * pixel_size;
        for y in 0..height {
            for x in 0..width {
                let pixel = pixels[x + y * width];

                //Convert rgba to slice format. There's only support for rgba at the moment.
                let rgba_array: [u8; 4] = [
                    (pixel & 0xFF) as u8,
                    (pixel >> 8 & 0xFF) as u8,
                    (pixel >> 16 & 0xFF) as u8,
                    (pixel >> 24 & 0xFF) as u8,
                ];

                //Find where the pixel data should go.
                let dest_begin = (dest_row_size * (y + offsetY as usize))
                    + ((x + offsetX as usize) * pixel_size);

                let dest_end = dest_begin + pixel_size;
                //Copy/paste pixel data to target image.
                let dest_row_slice = &mut gl_texture.pixels[dest_begin..dest_end];
                dest_row_slice.copy_from_slice(&rgba_array[0..pixel_size]);
            }
        }

        wm.display.queue.write_texture(
            gl_texture
                .bindable_texture
                .as_ref()
                .unwrap()
                .tv
                .texture
                .as_image_copy(),
            &gl_texture.pixels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(gl_texture.width as u32 * 4),
                rows_per_image: Some(gl_texture.height as u32),
            },
            Extent3d {
                width: gl_texture.width as u32,
                height: gl_texture.height as u32,
                depth_or_array_layers: 1,
            },
        );
    };

    let tx = &TASK_CHANNELS.0;

    tx.send(Box::new(task)).unwrap();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getMaxTextureSize(_env: JNIEnv, _class: JClass) -> jint {
    let wm = RENDERER.get().unwrap();
    wm.display.adapter.limits().max_texture_dimension_2d as i32
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getWindowWidth(_env: JNIEnv, _class: JClass) -> jint {
    RENDERER
        .get()
        .map_or(1280, |wm| wm.display.config.read().width as i32)
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getWindowHeight(_env: JNIEnv, _class: JClass) -> jint {
    RENDERER
        .get()
        .map_or(720, |wm| wm.display.config.read().height as i32)
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn clearColor(_env: JNIEnv, _class: JClass, r: jfloat, g: jfloat, b: jfloat) {
    CLEAR_COLOR.store(Arc::new([r, g, b]));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn attachTextureBindGroup(_env: JNIEnv, _class: JClass, slot: jint, id: jint) {
    GL_COMMANDS
        .write()
        .0
        .push(GLCommand::AttachTexture(slot as u32, id));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn wmUsePipeline(_env: JNIEnv, _class: JClass, pipeline: jint) {
    GL_COMMANDS
        .write()
        .0
        .push(GLCommand::UsePipeline(pipeline as usize));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getVideoMode(env: JNIEnv, _class: JClass) -> jstring {
    let video_mode = RENDERER
        .get()
        .unwrap()
        .display
        .window
        .current_monitor()
        .unwrap()
        .video_modes()
        .find(|_| true)
        .unwrap();
    env.new_string(format!(
        "{}x{}@{}:{}",
        video_mode.size().width,
        video_mode.size().height,
        video_mode.refresh_rate_millihertz() / 1000,
        video_mode.bit_depth()
    ))
    .unwrap()
    .into_raw()
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setProjectionMatrix(mut env: JNIEnv, _class: JClass, float_array: JFloatArray) {
    let elements: AutoElements<jfloat> =
        unsafe { env.get_array_elements(&float_array, ReleaseMode::NoCopyBack) }.unwrap();

    let slice = unsafe { slice::from_raw_parts(elements.as_ptr(), elements.len()) };

    let mut cursor = Cursor::new(bytemuck::cast_slice::<f32, u8>(slice));
    let mut converted = Vec::with_capacity(slice.len());

    for _ in 0..slice.len() {
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    let slice_4x4: [[f32; 4]; 4] = *bytemuck::from_bytes(bytemuck::cast_slice(&converted));

    let matrix = Mat4::from_cols_array_2d(&slice_4x4);

    GL_COMMANDS.write().0.push(GLCommand::SetMatrix(matrix));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn drawIndexed(_env: JNIEnv, _class: JClass, count: jint) {
    GL_COMMANDS
        .write()
        .0
        .push(GLCommand::DrawIndexed(count as u32));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setVertexBuffer(env: JNIEnv, _class: JClass, byte_array: JByteArray) {
    let mut bytes = vec![0; env.get_array_length(&byte_array).unwrap() as usize];
    env.get_byte_array_region(&byte_array, 0, &mut bytes[..])
        .unwrap();

    let byte_slice = bytemuck::cast_slice(&bytes);
    let mut cursor = Cursor::new(byte_slice);
    let mut converted = Vec::with_capacity(bytes.len() / 4);

    assert_eq!(bytes.len() % 4, 0);

    for _ in 0..bytes.len() / 4 {
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    GL_COMMANDS
        .write()
        .0
        .push(GLCommand::SetVertexBuffer(Vec::from(bytemuck::cast_slice(
            &converted,
        ))));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setIndexBuffer(env: JNIEnv, _class: JClass, int_array: JIntArray) {
    let mut indices = vec![0; env.get_array_length(&int_array).unwrap() as usize];
    env.get_int_array_region(&int_array, 0, &mut indices[..])
        .unwrap();

    let slice = unsafe { slice::from_raw_parts(indices.as_ptr() as *mut u32, indices.len()) };

    GL_COMMANDS
        .write()
        .0
        .push(GLCommand::SetIndexBuffer(Vec::from(slice)));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setCursorPosition(_env: JNIEnv, _class: JClass, x: f64, y: f64) {
    RENDERER
        .get()
        .unwrap()
        .display
        .window
        .set_cursor_position(PhysicalPosition { x, y })
        .unwrap();
}

const GLFW_CURSOR_NORMAL: i32 = 212993;
const GLFW_CURSOR_HIDDEN: i32 = 212994;
const GLFW_CURSOR_DISABLED: i32 = 212995;

/// See https://www.glfw.org/docs/3.3/input_guide.html#cursor_mode
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setCursorMode(_env: JNIEnv, _class: JClass, mode: i32) {
    let window = &RENDERER.get().unwrap().display.window;
    match mode {
        GLFW_CURSOR_NORMAL => {
            window
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap();
            window.set_cursor_visible(true);
        }
        GLFW_CURSOR_HIDDEN => {
            window
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap();
            window.set_cursor_visible(false);
        }
        GLFW_CURSOR_DISABLED => {
            window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| {
                    window
                        .set_cursor_grab(CursorGrabMode::Locked)
                })
                .unwrap();
            window.set_cursor_visible(false);
        }
        _ => {
            log::warn!("Set cursor mode had an invalid mode.")
        }
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bindStarData(
    env: JNIEnv,
    _class: JClass,
    length: jint,
    int_array: JIntArray,
    byte_array: JByteArray,
) {
    let mut indices = vec![0; env.get_array_length(&int_array).unwrap() as usize];
    env.get_int_array_region(&int_array, 0, &mut indices[..])
        .unwrap();

    let mut bytes = vec![0; env.get_array_length(&byte_array).unwrap() as usize];
    env.get_byte_array_region(&byte_array, 0, &mut bytes[..])
        .unwrap();

    let byte_slice = bytemuck::cast_slice(&bytes);
    let mut cursor = Cursor::new(byte_slice);
    let mut converted = Vec::with_capacity(bytes.len() / 4);

    assert_eq!(bytes.len() % 4, 0);

    for _ in 0..bytes.len() / 4 {
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    //spawn a thread bc renderer wouldn't be initialized quite yet
    THREAD_POOL.spawn(move || loop {
        if RENDERER.get().is_none() {
            continue;
        }

        // *RENDERER.get().unwrap().mc.stars_length.write() = length as u32;

        // let mut index_buffer = RENDERER.get().unwrap().mc.stars_index_buffer.write();

        // *index_buffer = Some(
        //     RENDERER
        //         .get()
        //         .unwrap()
        //         .wgpu_state
        //         .device
        //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //             label: None,
        //             contents: bytemuck::cast_slice(&indices),
        //             usage: wgpu::BufferUsages::INDEX,
        //         }),
        // );

        // let mut vertex_buffer = RENDERER.get().unwrap().mc.stars_vertex_buffer.write();

        // *vertex_buffer = Some(
        //     RENDERER
        //         .get()
        //         .unwrap()
        //         .wgpu_state
        //         .device
        //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //             label: None,
        //             contents: bytemuck::cast_slice(&converted),
        //             usage: wgpu::BufferUsages::VERTEX,
        //         }),
        // );

        break;
    });
}
