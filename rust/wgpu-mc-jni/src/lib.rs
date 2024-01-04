pub extern crate wgpu_mc;

use core::slice;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::mem::size_of;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::{mem, thread};

use arc_swap::ArcSwap;
use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::Matrix4;
use crossbeam_channel::{unbounded, Receiver, Sender};
use jni::objects::{
    AutoElements, GlobalRef, JByteArray, JClass, JFloatArray, JIntArray, JObject, JString, JValue,
    ReleaseMode,
};
use jni::sys::{jboolean, jbyte, jbyteArray, jfloat, jint, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use rayon::{ThreadPool, ThreadPoolBuilder};
use wgpu::Extent3d;
use wgpu_mc::wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::window::{CursorGrabMode, Window};

use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{
    BlockStateProvider, Chunk, ChunkPos, LightLevel, CHUNK_HEIGHT, CHUNK_SECTION_HEIGHT,
    SECTIONS_PER_CHUNK,
};
use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::texture::{BindableTexture, TextureSamplerView};
use wgpu_mc::wgpu;
use wgpu_mc::wgpu::ImageDataLayout;
use wgpu_mc::{HasWindowSize, WindowSize, WmRenderer};

use crate::gl::{GLCommand, GlTexture, GL_ALLOC, GL_COMMANDS};
use crate::lighting::DeserializedLightData;
use crate::palette::{IdList, JavaPalette, PALETTE_STORAGE};
use crate::pia::{PackedIntegerArray, PIA_STORAGE};
use crate::settings::Settings;

mod alloc;
pub mod entity;
mod gl;
mod lighting;
mod palette;
mod pia;
mod renderer;
mod settings;

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
static WINDOW: OnceCell<Arc<Window>> = OnceCell::new();
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
    Lazy::new(|| ThreadPoolBuilder::new().num_threads(0).build().unwrap());

static CHUNKS: Lazy<RwLock<HashMap<ChunkPos, Arc<ChunkHolder>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

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

#[derive(Debug)]
struct ChunkHolder {
    pub sections: [Option<(JavaPalette, PackedIntegerArray)>; 24],
    pub light_data: Option<DeserializedLightData>,
}

#[derive(Debug)]
struct MinecraftBlockstateProvider<'a> {
    pub center: &'a ChunkHolder,
    pub west: Option<&'a ChunkHolder>,
    pub north: Option<&'a ChunkHolder>,
    pub south: Option<&'a ChunkHolder>,
    pub east: Option<&'a ChunkHolder>,

    pub pos: ChunkPos,

    pub air: BlockstateKey,
}

impl<'a> BlockStateProvider for MinecraftBlockstateProvider<'a> {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        puffin::profile_scope!("get state");

        //Minecraft technically has negative y values now, but chunk data is indexed [0,384} instead of [-64,256}
        if y >= CHUNK_HEIGHT as i16 || y < 0 {
            return ChunkBlockState::Air;
        }

        let chunk_x = (x >> 4) - self.pos[0];
        let chunk_z = (z >> 4) - self.pos[1];

        let chunk_option = match [chunk_x, chunk_z] {
            [0, 0] => Some(self.center),
            [0, -1] => self.north,
            [0, 1] => self.south,
            [1, 0] => self.east,
            [-1, 0] => self.west,
            _pos => return ChunkBlockState::Air,
        };

        let chunk = match chunk_option {
            None => return ChunkBlockState::Air,
            Some(chunk) => chunk,
        };

        let storage_index = (y / 16) as usize;

        let (palette, storage) = match &chunk.sections[storage_index] {
            Some(section) => section,
            None => return ChunkBlockState::Air,
        };

        let palette_key = storage.get(x, y as i32, z);
        let (_, block) = palette.get(palette_key as usize).unwrap();

        if *block == self.air {
            ChunkBlockState::Air
        } else {
            ChunkBlockState::State(*block)
        }
    }

    fn get_light_level(&self, x: i32, y: i16, z: i32) -> LightLevel {
        let chunk_x = (x >> 4) - self.pos[0];
        let chunk_z = (z >> 4) - self.pos[1];

        let chunk_option = match [chunk_x, chunk_z] {
            [0, 0] => Some(self.center),
            [0, -1] => self.north,
            [0, 1] => self.south,
            [1, 0] => self.east,
            [-1, 0] => self.west,
            _pos => return LightLevel::from_sky_and_block(0, 0),
        };

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
        let local_y = y as i32 & 0b1111;
        let local_z = z & 0b1111;

        let packed_coords = ((local_y << 8) | (local_z << 4) | (local_x)) as usize;

        let shift = (packed_coords & 1) << 2;

        let section = y as usize / CHUNK_SECTION_HEIGHT;

        let array_index = packed_coords >> 1;
        let absolute_index = (section * 2048) + array_index;

        let sky_light = (light_data.sky_light[absolute_index] >> shift) & 0b1111;
        let block_light = (light_data.block_light[absolute_index] >> shift) & 0b1111;

        LightLevel::from_sky_and_block(sky_light, block_light)
    }

    fn is_section_empty(&self, index: usize) -> bool {
        if index >= SECTIONS_PER_CHUNK {
            return true;
        }

        self.center.sections[index].is_none()
    }
}

struct WinitWindowWrapper<'a> {
    window: &'a Window,
}

impl HasWindowSize for WinitWindowWrapper<'_> {
    fn get_window_size(&self) -> WindowSize {
        WindowSize {
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
        }
    }
}

unsafe impl HasRawWindowHandle for WinitWindowWrapper<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for WinitWindowWrapper<'_> {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

struct MinecraftResourceManagerAdapter {
    jvm: JavaVM,
}

impl ResourceProvider for MinecraftResourceManagerAdapter {
    fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>> {
        let mut env = self.jvm.attach_current_thread().unwrap();

        let path = env.new_string(&id.0).unwrap();

        let bytes: JByteArray = env
            .call_static_method(
                "dev/birb/wgpu/rust/WgpuResourceProvider",
                "getResource",
                "(Ljava/lang/String;)[B",
                &[JValue::Object(&path.into())],
            )
            .ok()?
            .l()
            .ok()?
            .into();

        let elements: AutoElements<jbyte> =
            unsafe { env.get_array_elements(&bytes, ReleaseMode::NoCopyBack) }.ok()?;

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
        THREAD_POOL.spawn(|| {
            let mut guard = SETTINGS.write();
            *guard = Some(settings);
        });
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

    THREAD_POOL.spawn(|| {
        let mut write = SETTINGS.write();
        *write = Some(Settings::load_or_default());
    });
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
pub fn createChunk(
    _env: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    palettes_ptr: jlong,
    storages_ptr: jlong,
    block_light_ptr: jlong,
    sky_light_ptr: jlong,
) {
    let palettes = unsafe { &*(palettes_ptr as usize as *mut [usize; SECTIONS_PER_CHUNK]) };

    let storages = unsafe { &*(storages_ptr as usize as *mut [usize; SECTIONS_PER_CHUNK]) };

    let block_light =
        unsafe { (block_light_ptr as usize as *mut [u8; 2048 * SECTIONS_PER_CHUNK]).read() };

    let sky_light =
        unsafe { (sky_light_ptr as usize as *mut [u8; 2048 * SECTIONS_PER_CHUNK]).read() };

    assert_eq!(size_of::<usize>(), 8);

    let holder = ChunkHolder {
        sections: palettes
            .iter()
            .zip(storages.iter())
            .map(|(&palette, &storage)| {
                if palette == 0 || storage == 0 {
                    return None;
                }

                //The indices are incremented by one in Java so that 0 means null/None
                Some((
                    PALETTE_STORAGE.read().get(palette - 1).unwrap().clone(),
                    PIA_STORAGE.read().get(storage - 1).unwrap().clone(),
                ))
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("Expected a Vec of length 24, got {}", palettes.len())),
        light_data: Some(DeserializedLightData {
            block_light,
            sky_light,
        }),
    };

    let mut write = CHUNKS.write();

    write.insert([x, z], Arc::new(holder));
}

pub fn bake_chunk(x: i32, z: i32) {
    puffin::profile_scope!("jni bake chunk", format!("{},{}", x, z));

    let wm = RENDERER.get().unwrap();

    let bm = wm.mc.block_manager.read();

    {
        if !wm.mc.chunks.loaded_chunks.contains_key(&[x, z]) {
            wm.mc
                .chunks
                .loaded_chunks
                .insert([x, z], ArcSwap::new(Arc::new(Chunk::new([x, z]))));
        }
    }

    let (chunk, center, neighbors) = {
        let chunk = wm.mc.chunks.loaded_chunks.get(&[x, z]).unwrap().load_full();
        let chunks = CHUNKS.read();

        let center = chunks.get(&[x, z]).unwrap().clone();

        let neighbors = [
            chunks.get(&[x, z - 1]), //North
            chunks.get(&[x, z + 1]), //South
            chunks.get(&[x - 1, z]), //West
            chunks.get(&[x + 1, z]), //East
        ]
        .map(|arc_option| arc_option.map(|arc| arc.clone()));

        (chunk, center, neighbors)
    };

    let bsp = MinecraftBlockstateProvider {
        center: &center,
        north: neighbors[0].as_ref().map(|arc| &**arc),
        south: neighbors[1].as_ref().map(|arc| &**arc),
        west: neighbors[2].as_ref().map(|arc| &**arc),
        east: neighbors[3].as_ref().map(|arc| &**arc),
        pos: [x, z],
        air: *AIR,
    };

    chunk.bake_chunk(wm, &wm.pipelines.load_full().chunk_layers.load(), &bm, &bsp);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn clearChunks(_env: JNIEnv, _class: JClass) {
    THREAD_POOL.spawn(|| {
        let wm = RENDERER.get().unwrap();

        wm.mc.chunks.loaded_chunks.clear();
    });
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bakeChunk(_env: JNIEnv, _class: JClass, x: jint, z: jint) {
    bake_chunk(x, z);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerBlock(mut env: JNIEnv, _class: JClass, name: JString) {
    let name: String = env.get_string(&name).unwrap().into();

    BLOCKS.lock().push(name);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn startRendering(env: JNIEnv, _class: JClass, title: JString) {
    renderer::start_rendering(env, title);
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

            let atlas = wm
                .mc
                .texture_manager
                .atlases
                .load()
                .get(BLOCK_ATLAS)
                .unwrap()
                .load_full();

            let model = wm_block.get_model_by_key(
                key_iter
                    .iter()
                    .filter(|(a, _)| *a != "waterlogged")
                    .map(|(a, b)| (*a, b)),
                &*wm.mc.resource_provider,
                &atlas,
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

    thread::spawn(|| {
        let rx = &TASK_CHANNELS.1;
        for task in rx.iter() {
            task()
        }
    });

    let rx = &CHANNELS.1;

    for render_message in rx.iter() {
        match render_message {
            RenderMessage::SetTitle(title) => WINDOW.get().unwrap().set_title(&title),
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
    if let Some(window) = WINDOW.get() {
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
    if let Some(window) = WINDOW.get() {
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

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn digestInputStream(mut env: JNIEnv, _class: JClass, input_stream: JObject) -> jbyteArray {
    let mut vec = Vec::with_capacity(1024);
    let array = env.new_byte_array(1024).unwrap();

    loop {
        let bytes_read = env
            .call_method(&input_stream, "read", "([B)I", &[JValue::Object(&array)])
            .unwrap()
            .i()
            .unwrap();

        //bytes_read being -1 means EOF
        if bytes_read > 0 {
            let elements =
                unsafe { env.get_array_elements(&array, ReleaseMode::NoCopyBack) }.unwrap();

            let slice: &[u8] = unsafe {
                mem::transmute(slice::from_raw_parts(
                    elements.as_ptr(),
                    bytes_read as usize,
                ))
            };

            vec.extend_from_slice(slice);
        } else {
            break;
        }
    }

    let bytes = env.new_byte_array(vec.len() as i32).unwrap();
    let bytes_elements = unsafe { env.get_array_elements(&bytes, ReleaseMode::CopyBack) }.unwrap();

    unsafe {
        std::ptr::copy(vec.as_ptr(), bytes_elements.as_ptr() as *mut u8, vec.len());
    }

    bytes.as_raw()
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

        let tsv = TextureSamplerView::from_rgb_bytes(
            &wm.wgpu_state,
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

        let bindable =
            BindableTexture::from_tsv(&wm.wgpu_state, &wm.pipelines.load_full(), tsv, false);

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

        wm.wgpu_state.queue.write_texture(
            gl_texture
                .bindable_texture
                .as_ref()
                .unwrap()
                .tsv
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
    wm.wgpu_state.adapter.limits().max_texture_dimension_2d as i32
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getWindowWidth(_env: JNIEnv, _class: JClass) -> jint {
    RENDERER
        .get()
        .map_or(1280, |wm| wm.wgpu_state.surface.read().1.width as i32)
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getWindowHeight(_env: JNIEnv, _class: JClass) -> jint {
    RENDERER
        .get()
        .map_or(720, |wm| wm.wgpu_state.surface.read().1.height as i32)
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
    let video_mode = WINDOW
        .get()
        .unwrap()
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

    let matrix = Matrix4::from(slice_4x4);

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
pub fn createIdList(_env: JNIEnv, _class: JClass) -> jlong {
    let mut palette = Box::new(IdList::new());

    let ptr = ((&mut *palette as *mut IdList) as usize) as jlong;
    mem::forget(palette);

    ptr
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn addIdListEntry(
    env: JNIEnv,
    _class: JClass,
    idlist_long: jlong,
    index: jint,
    object: JObject,
) {
    let idlist = (idlist_long as usize) as *mut IdList;

    unsafe {
        idlist
            .as_mut()
            .unwrap()
            .map
            .insert(index, env.new_global_ref(object).unwrap())
    };
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setCursorPosition(_env: JNIEnv, _class: JClass, x: f64, y: f64) {
    WINDOW
        .get()
        .unwrap()
        .set_cursor_position(PhysicalPosition { x, y })
        .unwrap();
}

const GLFW_CURSOR_NORMAL: i32 = 212993;
const GLFW_CURSOR_HIDDEN: i32 = 212994;
const GLFW_CURSOR_DISABLED: i32 = 212995;

/// See https://www.glfw.org/docs/3.3/input_guide.html#cursor_mode
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setCursorMode(_env: JNIEnv, _class: JClass, mode: i32) {
    match mode {
        GLFW_CURSOR_NORMAL => {
            WINDOW
                .get()
                .unwrap()
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap();
            WINDOW.get().unwrap().set_cursor_visible(true);
        }
        GLFW_CURSOR_HIDDEN => {
            WINDOW
                .get()
                .unwrap()
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap();
            WINDOW.get().unwrap().set_cursor_visible(false);
        }
        GLFW_CURSOR_DISABLED => {
            WINDOW
                .get()
                .unwrap()
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| {
                    WINDOW
                        .get()
                        .unwrap()
                        .set_cursor_grab(CursorGrabMode::Locked)
                })
                .unwrap();
            WINDOW.get().unwrap().set_cursor_visible(false);
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

        *RENDERER.get().unwrap().mc.stars_length.write() = length as u32;

        let mut index_buffer = RENDERER.get().unwrap().mc.stars_index_buffer.write();

        *index_buffer = Some(
            RENDERER
                .get()
                .unwrap()
                .wgpu_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        );

        let mut vertex_buffer = RENDERER.get().unwrap().mc.stars_vertex_buffer.write();

        *vertex_buffer = Some(
            RENDERER
                .get()
                .unwrap()
                .wgpu_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&converted),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        );

        break;
    });
}
