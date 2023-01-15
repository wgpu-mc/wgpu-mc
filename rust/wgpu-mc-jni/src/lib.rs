#![feature(once_cell)]
#![feature(array_zip)]
#![feature(core_panic)]

use core::slice;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::Cursor;
use std::{mem, thread};
use std::mem::size_of;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::gl::{GLCommand, GlTexture, GL_ALLOC, GL_COMMANDS};
use arc_swap::ArcSwap;
use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Matrix4, Point3};
use crossbeam_channel::{unbounded, Receiver, Sender};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue, ReleaseMode};
use jni::sys::{
    jboolean, jbyteArray, jdouble, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray,
    jstring, JNI_FALSE, JNI_TRUE,
};
use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use rayon::{ThreadPool, ThreadPoolBuilder};
use wgpu::Extent3d;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::window::{CursorGrabMode, Window};

use entity::TexturedModelData;
use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, ChunkPos, CHUNK_HEIGHT, CHUNK_SECTIONS_PER};
use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;
use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::texture::{BindableTexture, TextureSamplerView};
use wgpu_mc::wgpu;
use wgpu_mc::wgpu::ImageDataLayout;
use wgpu_mc::{HasWindowSize, WindowSize, WmRenderer};

use crate::entity::tmd_to_wm;
use crate::palette::{IdList, JavaPalette, PALETTE_STORAGE};
use crate::pia::{PackedIntegerArray, PIA_STORAGE};
use crate::settings::Settings;

mod entity;
mod gl;
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

static CHANNELS: Lazy<(Sender<RenderMessage>, Receiver<RenderMessage>)> = Lazy::new(unbounded);
static TASK_CHANNELS: Lazy<(Sender<Box<dyn FnOnce() + Send + Sync>>, Receiver<Box<dyn FnOnce() + Send + Sync>>)> = Lazy::new(unbounded);
static MC_STATE: Lazy<ArcSwap<MinecraftRenderState>> = Lazy::new(|| {
    ArcSwap::new(Arc::new(MinecraftRenderState {
        _render_world: false,
    }))
});
#[allow(dead_code)]
static MOUSE_STATE: Lazy<Arc<ArcSwap<MouseState>>> =
    Lazy::new(|| Arc::new(ArcSwap::new(Arc::new(MouseState { x: 0.0, y: 0.0 }))));
static THREAD_POOL: Lazy<ThreadPool> =
    Lazy::new(|| ThreadPoolBuilder::new().num_threads(0).build().unwrap());

static CHUNKS: Lazy<RwLock<HashMap<ChunkPos, ChunkHolder>>> =
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

    fn is_section_empty(&self, index: usize) -> bool {
        if index >= CHUNK_SECTIONS_PER {
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
        let env = self.jvm.attach_current_thread().unwrap();

        let path = env.new_string(&id.0).unwrap();

        let bytes = env
            .call_static_method(
                "dev/birb/wgpu/rust/WgpuResourceProvider",
                "getResource",
                "(Ljava/lang/String;)[B",
                &[JValue::Object(path.into())],
            )
            .ok()?
            .l()
            .ok()?;

        let elements = env
            .get_byte_array_elements(bytes.into_raw(), ReleaseMode::NoCopyBack)
            .ok()?;

        let size = elements.size().ok()? as usize;

        let _vec = vec![0u8; size];

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
pub fn sendSettings(env: JNIEnv, _class: JClass, settings: JString) -> bool {
    let json: String = env.get_string(settings).unwrap().into();
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
pub fn sendRunDirectory(env: JNIEnv, _class: JClass, dir: JString) {
    let dir: String = env.get_string(dir).unwrap().into();
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
    env: JNIEnv,
    _class: JClass,
    block_state: JObject,
    block_name: JString,
    state_key: JString,
) {
    let global_ref = env.new_global_ref(block_state).unwrap();

    let block_name: String = env.get_string(block_name).unwrap().into();
    let state_key: String = env.get_string(state_key).unwrap().into();

    BLOCK_STATES
        .lock()
        .push((block_name, state_key, global_ref));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createChunk(
    env: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    palettes: jlongArray,
    storages: jlongArray,
) {
    let palette_elements = env
        .get_long_array_elements(palettes, ReleaseMode::NoCopyBack)
        .unwrap();

    let storage_elements = env
        .get_long_array_elements(storages, ReleaseMode::NoCopyBack)
        .unwrap();

    let palette_elements = unsafe {
        slice::from_raw_parts(
            palette_elements.as_ptr(),
            palette_elements.size().unwrap() as usize,
        )
    };

    let palettes: &[usize; 24] = bytemuck::cast_slice::<_, usize>(palette_elements)
        .try_into()
        .unwrap();

    let storage_elements = unsafe {
        slice::from_raw_parts(
            storage_elements.as_ptr(),
            storage_elements.size().unwrap() as usize,
        )
    };

    assert_eq!(size_of::<usize>(), 8);

    let storages: &[usize; 24] = bytemuck::cast_slice::<_, usize>(storage_elements)
        .try_into()
        .unwrap();

    let mut write = CHUNKS.write();

    write.insert(
        [x, z],
        ChunkHolder {
            sections: palettes.zip(*storages).map(|(palette, storage)| {
                if palette == 0 || storage == 0 {
                    return None;
                }

                //The indices are incremented by one in Java so that 0 means null/None
                Some((
                    PALETTE_STORAGE.read().get(palette - 1).unwrap().clone(),
                    PIA_STORAGE.read().get(storage - 1).unwrap().clone(),
                ))
            }),
        },
    );
}

pub fn bake_chunk(x: i32, z: i32) {
    THREAD_POOL.spawn(move || {
        let wm = RENDERER.get().unwrap();

        {
            {
                let loaded_chunks = wm.mc.chunks.loaded_chunks.read();
                if !loaded_chunks.contains_key(&[x, z]) {
                    drop(loaded_chunks);
                    let mut loaded_chunks = wm.mc.chunks.loaded_chunks.write();
                    loaded_chunks.insert([x, z], ArcSwap::new(Arc::new(Chunk::new([x, z]))));
                }
            }

            let bm = wm.mc.block_manager.read();
            let loaded_chunks = wm.mc.chunks.loaded_chunks.read();

            let chunk = loaded_chunks.get(&[x, z]).unwrap().load();

            let chunks = CHUNKS.read();

            let center = chunks.get(&[x, z]).unwrap();
            let north = chunks.get(&[x, z - 1]);
            let south = chunks.get(&[x, z + 1]);
            let west = chunks.get(&[x - 1, z]);
            let east = chunks.get(&[x + 1, z]);

            let bsp = MinecraftBlockstateProvider {
                center,
                west,
                north,
                south,
                east,
                pos: [x, z],
                air: *AIR,
            };

            let instant = Instant::now();

            chunk.bake(wm, &wm.pipelines.load_full().chunk_layers.load(), &bm, &bsp);
        }
    });
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn clearChunks(_env: JNIEnv, _class: JClass) {
    THREAD_POOL.spawn(|| {
        let wm = RENDERER.get().unwrap();

        let mut chunks = wm.mc.chunks.loaded_chunks.write();
        chunks.clear();
    });
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bakeChunk(_env: JNIEnv, _class: JClass, x: jint, z: jint) {
    bake_chunk(x, z);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerBlock(env: JNIEnv, _class: JClass, name: JString) {
    let name: String = env.get_string(name).unwrap().into();

    println!("{name}");

    BLOCKS.lock().push(name);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn startRendering(env: JNIEnv, _class: JClass, title: JString) {
    renderer::start_rendering(env, title);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn cacheBlockStates(env: JNIEnv, _class: JClass) {
    let wm = RENDERER.get().unwrap();
    {
        let blocks = BLOCKS.lock();

        let blockstates = blocks
            .iter()
            .map(|identifier| {
                (
                    identifier.clone(),
                    ResourcePath::try_from(&identifier[..])
                        .unwrap()
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

    let states = BLOCK_STATES.lock();

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
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn runHelperThread(env: JNIEnv, _class: JClass) {
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
                    MouseButton::Other(_) => 0,
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
pub fn centerCursor(env: JNIEnv, _class: JClass, locked: jboolean) {
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
pub fn setCursorLocked(env: JNIEnv, _class: JClass, locked: jboolean) {
    if let Some(window) = WINDOW.get() {
        window.set_cursor_grab(match locked {
            JNI_TRUE => {
                window.set_cursor_visible(false);
                CursorGrabMode::Confined
            }
            JNI_FALSE => {
                window.set_cursor_visible(true);
                CursorGrabMode::None
            }
            _ => unreachable!(),
        });
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
        let env = jvm.attach_current_thread_permanently().unwrap();

        let message = format!("wgpu-mc has panicked. The JVM will now exit.\n{panic_info}");
        let jstring = env.new_string(message).unwrap();

        //Does not return
        env.call_static_method(
            "dev/birb/wgpu/render/Wgpu",
            "rustPanic",
            "(Ljava/lang/String;)V",
            &[JValue::Object(unsafe {
                JObject::from_raw(jstring.into_raw())
            })],
        );
    }))
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn debugBake(env: JNIEnv, _class: JClass) {
    let positions = {
        let renderer = RENDERER.get().unwrap();
        let chunks = renderer.mc.chunks.loaded_chunks.read();

        chunks.iter().map(|(pos, _)| *pos).collect::<Vec<_>>()
    };

    println!("Baking {0} chunks", positions.len());
    for pos in positions {
        bake_chunk(pos[0], pos[1]);
    }

    // let wm = RENDERER.get().unwrap();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn digestInputStream(env: JNIEnv, _class: JClass, input_stream: JObject) -> jbyteArray {
    let mut vec = Vec::with_capacity(1024);
    let array = env.new_byte_array(1024).unwrap();

    loop {
        let bytes_read = env
            .call_method(
                input_stream,
                "read",
                "([B)I",
                &[unsafe { JObject::from_raw(array) }.into()],
            )
            .unwrap()
            .i()
            .unwrap();

        //bytes_read being -1 means EOF
        if bytes_read > 0 {
            let elements = env
                .get_byte_array_elements(array, ReleaseMode::NoCopyBack)
                .unwrap();

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
    let bytes_elements = env
        .get_byte_array_elements(bytes, ReleaseMode::CopyBack)
        .unwrap();

    unsafe {
        std::ptr::copy(vec.as_ptr(), bytes_elements.as_ptr() as *mut u8, vec.len());
    }

    bytes
}

#[allow(unused_must_use)]
#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn updateWindowTitle(env: JNIEnv, _class: JClass, jtitle: JString) {
    let tx = &CHANNELS.0;

    let title: String = env.get_string(jtitle).unwrap().into();

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

    std::mem::swap(command_stack, submitted);

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
    _env: JNIEnv,
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
    pixels: jlong,
    unpack_row_length: jint,
    unpack_skip_pixels: jint,
    unpack_skip_rows: jint,
    unpack_alignment: jint,
) {
    let mut pixels = pixels as usize;
    let unpack_row_length = unpack_row_length as usize;
    let unpack_skip_pixels = unpack_skip_pixels as usize;
    let unpack_skip_rows = unpack_skip_rows as usize;
    let unpack_alignment = unpack_alignment as usize;
    let width = width as usize;
    let height = height as usize;

    let pixel_size = match format {
        0x1908 | 0x80E1 => 4,
        _ => panic!("Unknown format {format:x}"),
    };

    //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glPixelStore.xhtml
    let row_width = if unpack_row_length > 0 {
        unpack_row_length as i64
    } else {
        width as i64
    };

    let src_row_size = row_width as usize * pixel_size;

    //GL_UNPACK_SKIP_PIXELS
    pixels += pixel_size * unpack_skip_pixels;
    //GL_UNPACK_SKIP_ROWS
    pixels += src_row_size * unpack_skip_rows;

    let next_row_byte_offset = if pixel_size >= unpack_alignment {
        src_row_size
    } else {
        unimplemented!()
    };

    //In bytes
    assert_eq!(_type, 0x1401);

    let vec = unsafe {
        Vec::from(slice::from_raw_parts(
            pixels as *mut u8,
            next_row_byte_offset * height,
        ))
    };

    //For when the renderer is initialized
    let task = move || {
        let wm = RENDERER.get().unwrap();

        let mut alloc_write = GL_ALLOC.write();

        let gl_texture = alloc_write.get_mut(&(texture_id as u32)).unwrap();

        let dest_row_size = gl_texture.width as usize * pixel_size;

        let mut pixel_offset = 0usize;
        for y in 0..height {
            let src_row_slice = &vec[pixel_offset..pixel_offset + src_row_size];
            pixel_offset += next_row_byte_offset;

            let dest_begin =
                (dest_row_size * (y + offsetY as usize)) + (offsetX as usize * pixel_size);
            let dest_end = dest_begin + src_row_size;

            let dest_row_slice = &mut gl_texture.pixels[dest_begin..dest_end];
            dest_row_slice.copy_from_slice(src_row_slice);
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
                bytes_per_row: NonZeroU32::new(gl_texture.width as u32 * 4),
                rows_per_image: NonZeroU32::new(gl_texture.height as u32),
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
    GL_COMMANDS.write().0.push(GLCommand::ClearColor([r, g, b]));
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
pub fn setProjectionMatrix(env: JNIEnv, _class: JClass, float_array: jfloatArray) {
    let elements = env
        .get_float_array_elements(float_array, ReleaseMode::NoCopyBack)
        .unwrap();

    let slice = unsafe {
        slice::from_raw_parts(
            elements.as_ptr() as *mut f32,
            elements.size().unwrap() as usize,
        )
    };

    let mut cursor = Cursor::new(bytemuck::cast_slice::<f32, u8>(slice));
    let mut converted = Vec::with_capacity(slice.len());

    for _ in 0..slice.len() {
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    let slice_4x4: [[f32; 4]; 4] = *bytemuck::from_bytes(bytemuck::cast_slice(&converted));

    let matrix = Matrix4::from(slice_4x4) * Matrix4::from_nonuniform_scale(1.0, 1.0, 0.0);

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
pub fn setVertexBuffer(env: JNIEnv, _class: JClass, byte_array: jbyteArray) {
    let mut bytes = vec![0; env.get_array_length(byte_array).unwrap() as usize];
    env.get_byte_array_region(byte_array, 0, &mut bytes[..])
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
pub fn setIndexBuffer(env: JNIEnv, _class: JClass, int_array: jintArray) {
    let elements = env
        .get_int_array_elements(int_array, ReleaseMode::NoCopyBack)
        .unwrap();

    let slice = unsafe {
        slice::from_raw_parts(
            elements.as_ptr() as *mut u32,
            elements.size().unwrap() as usize,
        )
    };

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
pub fn setCamera(
    _env: JNIEnv,
    _class: JClass,
    x: jdouble,
    _y: jdouble,
    z: jdouble,
    yaw: jfloat,
    pitch: jfloat,
) {
    // let renderer = RENDERER.get().unwrap();
    // if renderer.mc.camera_bind_group.load().is_none() {
    //     renderer.mc.init_camera(renderer);
    // }
    //
    // let mut camera = **renderer.mc.camera.load();
    // camera.position = Point3::new(x as f32, 200., z as f32);
    // // camera.position = Point3::new(0.0, 200.0, 0.0);
    // camera.yaw = (PI / 180.0) * yaw;
    // camera.pitch = (PI / 180.0) * pitch;
    // // camera.pitch = PI * 1.5;
    //
    // renderer.mc.camera.store(Arc::new(camera));
    // renderer.upload_camera();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerEntityModel(env: JNIEnv, _class: JClass, json_jstring: JString) {
    let _renderer = RENDERER.get().unwrap();

    let json_string: String = env.get_string(json_jstring).unwrap().into();
    let model_data: TexturedModelData = serde_json::from_str(&json_string).unwrap();
    let _entity_part = tmd_to_wm(&model_data.data.data);
}
