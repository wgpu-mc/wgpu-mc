#![feature(once_cell)]
#![feature(mixed_integer_ops)]

extern crate core;

use crate::gl::GlTexture;
use crate::palette::{IdList, JavaPalette};
use arc_swap::ArcSwap;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use cgmath::{Deg, Matrix4, Point3, Vector3};
use core::slice;
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::block_on;
use gl::pipeline::{GLCommand, GlPipeline};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue, ReleaseMode};
use jni::sys::{
    _jobject, jboolean, jbyte, jbyteArray, jdouble, jfloat, jfloatArray, jint, jintArray, jlong,
    jlongArray, jobject, jstring,
};
use jni::{JNIEnv, JavaVM};
use mc_varint::VarIntRead;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env::var;
use std::io::Cursor;
use std::num::NonZeroU32;
use std::ptr::drop_in_place;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{fs, mem, thread};
use std::f32::consts::PI;
use std::fmt::{Debug, Formatter};
use std::ops::Shr;
use rayon::{ThreadPool, ThreadPoolBuilder};
use wgpu::Extent3d;
use wgpu_mc::mc::block::{BlockPos, ChunkBlockState, BlockstateKey};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, CHUNK_HEIGHT};
use wgpu_mc::mc::resource::{ResourceProvider, ResourcePath};
use wgpu_mc::render::pipeline::terrain::TerrainPipeline;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::texture::{TextureSamplerView, BindableTexture};
use wgpu_mc::wgpu;
use wgpu_mc::wgpu::ImageDataLayout;
use wgpu_mc::{HasWindowSize, WindowSize, WmRenderer};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, ModifiersState, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;
use wgpu_mc::camera::Camera;

mod gl;
mod mc_interface;
mod palette;
enum RenderMessage {
    SetTitle(String),
    Task(Box<dyn FnOnce() + Send + Sync>),
    KeyPressed(u32),
    MouseState(ElementState, MouseButton),
    KeyState(u32, u32, u32, u32),
    CharTyped(char, u32),
    MouseMove(f64, f64),
    Resized,
}

struct MinecraftRenderState {
    //draw_queue: Vec<>,
    render_world: bool,
}

struct MouseState {
    pub x: f64,
    pub y: f64,
}

static RENDERER: OnceCell<WmRenderer> = OnceCell::new();
static CHANNELS: OnceCell<(Sender<RenderMessage>, Receiver<RenderMessage>)> = OnceCell::new();
static MC_STATE: OnceCell<ArcSwap<MinecraftRenderState>> = OnceCell::new();
static MOUSE_STATE: OnceCell<Arc<ArcSwap<MouseState>>> = OnceCell::new();
static GL_PIPELINE: OnceCell<GlPipeline> = OnceCell::new();
static WINDOW: OnceCell<Arc<Window>> = OnceCell::new();
static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();

static JAVA_BLOCK_STATES: OnceCell<RwLock<HashMap<String, GlobalRef>>> = OnceCell::new();
static BLOCKS: OnceCell<Mutex<Vec<String>>> = OnceCell::new();
static BLOCK_STATE_PROVIDER: OnceCell<MinecraftBlockstateProvider> = OnceCell::new();

#[derive(Debug)]
struct ChunkHolder {
    pub palettes: [usize; 24],
    pub storages: [usize; 24],
}

#[derive(Debug)]
struct MinecraftBlockstateProvider {
    pub chunks: RwLock<HashMap<(i32, i32), ChunkHolder>>,
    pub air: BlockstateKey
}

impl BlockStateProvider for MinecraftBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        //Minecraft technically has negative y values now, but chunk data is technically indexed [0,384} instead of [-64,256}
        if y >= CHUNK_HEIGHT as i16 || y < 0 {
            return ChunkBlockState::Air;
        }

        let chunk_x = x / 16;
        let chunk_z = z / 16;

        let chunks_read = self.chunks.read();

        let chunk = chunks_read.get(&(chunk_x, chunk_z)).unwrap();

        let storage_index = (y / 16) as usize;

        let storage = unsafe {
            (chunk.storages[storage_index as usize] as *mut PackedIntegerArray).as_ref()
        }.unwrap();

        let palette = unsafe {
            (chunk.palettes[storage_index] as *mut JavaPalette).as_ref()
        }.unwrap();

        let palette_key = storage.get(x, y as u16, z);
        let block = palette.get(palette_key as usize).unwrap().1;

        if block == self.air {
            return ChunkBlockState::Air;
        } else {
            return ChunkBlockState::State(block)
        }
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
                &[
                    JValue::Object(path.into())
                ],
            ).ok()?.l().ok()?;

        let elements = env
            .get_byte_array_elements(bytes.into_inner(), ReleaseMode::NoCopyBack).ok()?;

        let size = elements.size().ok()? as usize;

        let mut vec = vec![0u8; size];
        vec.copy_from_slice(unsafe {
            std::slice::from_raw_parts(
                elements.as_ptr() as *const u8,
                size,
            )
        });

        Some(vec)
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_uploadChunk(
    env: JNIEnv,
    _class: JClass,
    world_chunk: JObject,
) {
    mc_interface::chunk_from_java_world_chunk(&env, &world_chunk);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getBackend(
    env: JNIEnv,
    class: JClass,
) -> jstring {
    let renderer = RENDERER.get().unwrap();
    let backend = renderer.get_backend_description();

    env.new_string(backend).unwrap().into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_registerBlockState(
    env: JNIEnv,
    class: JClass,
    block_state: JObject,
    name: JString,
) {
    let global_ref = env.new_global_ref(block_state).unwrap();

    let block_state_key: String = env.get_string(name).unwrap().into();
    // let block_state_key: String = env.get_string(JString::from(env.call_method(real_ptr, "toString", "()Ljava/lang/String;", &[]).unwrap().l().unwrap())).unwrap().into();

    let mut java_block_states = JAVA_BLOCK_STATES
        .get_or_init(|| RwLock::new(HashMap::new()))
        .write();

    java_block_states.insert(block_state_key, global_ref);
}

//This is per chunk
#[derive(Debug)]
struct JavaBlockStateProvider {
    //Pointers to JavaPalette structs
    pub palettes: [usize; 24],
    //Pointers to PackedIntegerArray structs
    pub storages: [usize; 24],
    pub air: BlockstateKey
}

impl BlockStateProvider for JavaBlockStateProvider {
    //Technically this should be able to provide a blockstate for anywhere in the world but I won't implement that yet
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        if y >= CHUNK_HEIGHT as i16 || y < 0 {
            return ChunkBlockState::Air;
        }

        let storage_index = y / 16;

        assert!(storage_index < (CHUNK_HEIGHT as i16 / 16) && storage_index >= 0);

        let storage = match unsafe {
            (self.storages[storage_index as usize] as *mut PackedIntegerArray).as_ref()
        } {
            None => return ChunkBlockState::Air,
            Some(storage) => storage,
        };

        let palette_key = storage.get(x, y as u16, z);

        let palette = unsafe {
            (self.palettes[(y as usize) / 24] as *mut JavaPalette)
                .as_ref()
                .unwrap()
        };

        let key = palette.get(palette_key as usize).unwrap().1;

        if key == self.air {
            ChunkBlockState::Air
        } else {
            ChunkBlockState::State(key)
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createChunk(
    env: JNIEnv,
    class: JClass,
    x: jint,
    z: jint,
    palettes: jlongArray,
    storages: jlongArray,
) {
    let wm = RENDERER.get().unwrap();

    let palette_elements = env
        .get_long_array_elements(palettes, ReleaseMode::NoCopyBack)
        .unwrap();

    let storage_elements = env
        .get_long_array_elements(storages, ReleaseMode::NoCopyBack)
        .unwrap();

    let palette_elements = unsafe {
        std::slice::from_raw_parts(
            palette_elements.as_ptr(),
            palette_elements.size().unwrap() as usize,
        )
    };

    let palettes: &[usize; 24] = bytemuck::cast_slice::<_, usize>(palette_elements)
        .try_into()
        .unwrap();

    let storage_elements = unsafe {
        std::slice::from_raw_parts(
            storage_elements.as_ptr(),
            storage_elements.size().unwrap() as usize,
        )
    };

    let storages: &[usize; 24] = bytemuck::cast_slice::<_, usize>(storage_elements)
        .try_into()
        .unwrap();

    let jbsp = JavaBlockStateProvider {
        storages: *storages,
        palettes: *palettes,
        air: BlockstateKey {
            block: wm.mc.block_manager.read().blocks.get_full("minecraft:air").unwrap().0 as u16,
            augment: 0
        }
    };

    let chunk = Chunk {
        pos: (x, z),
        baked: ArcSwap::new(Arc::new(None))
    };

    wm.mc.chunks.loaded_chunks
        .write()
        .insert((x, z), ArcSwap::new(Arc::new(chunk)));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_bakeChunk(
    env: JNIEnv,
    class: JClass,
    x: jint,
    z: jint
) {
    THREAD_POOL.get().unwrap().spawn(move || {
        let wm = RENDERER.get().unwrap();
        let bm = wm.mc.block_manager.read();

        let chunk = wm.mc.chunks.loaded_chunks
            .read()
            .get(&(x, z))
            .unwrap().load();

        let instant = Instant::now();
        chunk.bake(&bm, BLOCK_STATE_PROVIDER.get().unwrap());

        println!(
            "Baked chunk (x={}, z={}) in {}ms",
            x,
            z,
            Instant::now().duration_since(instant).as_millis(),
        );

        wm.mc.chunks.assemble_world_meshes(&wm);
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_registerBlock(
    env: JNIEnv,
    class: JClass,
    name: JString,
) {
    let name: String = env.get_string(name).unwrap().into();

    println!("{}", name);

    BLOCKS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .push(name);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_startRendering(
    env: JNIEnv,
    _class: JClass,
    string: JString,
) {
    use winit::event_loop::EventLoop;

    let title: String = env.get_string(string).unwrap().into();

    THREAD_POOL.set(
        ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap()
    );

    // Hacky fix for starting the game on linux, needs more investigation (thanks, accusitive)
    #[cfg(target_os = "linux")]
    let event_loop: EventLoop<()> = winit::platform::unix::EventLoopExtUnix::new_any_thread();
    #[cfg(not(target_os = "linux"))]
    let event_loop: EventLoop<()> = EventLoop::new();

    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
                width: 1280,
                height: 720,
            }))
            .build(&event_loop)
            .unwrap(),
    );

    println!("Opened window");

    WINDOW.set(window.clone());

    MC_STATE.set(ArcSwap::new(Arc::new(MinecraftRenderState {
        render_world: false,
    })));

    let wrapper = &WinitWindowWrapper { window: &window };

    let wgpu_state = block_on(WmRenderer::init_wgpu(wrapper));

    println!("Initialized wgpu");

    let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
        jvm: env.get_java_vm().unwrap(),
    });

    let wm = WmRenderer::new(wgpu_state, resource_provider);

    RENDERER.set(wm.clone());

    wm.init(&[&TerrainPipeline, GL_PIPELINE.get().unwrap()]);

    println!("Initialized wgpu-mc pipelines");

    wm.mc.chunks.assemble_world_meshes(&wm);

    println!("Assembled meshes");

    env.set_static_field(
        "dev/birb/wgpu/render/Wgpu",
        ("dev/birb/wgpu/render/Wgpu", "INITIALIZED", "Z"),
        JValue::Bool(true.into()),
    );

    let mut current_modifiers = ModifiersState::empty();

    println!("Starting event loop");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        wm.resize(WindowSize {
                            width: physical_size.width,
                            height: physical_size.height,
                        });
                        CHANNELS
                            .get()
                            .unwrap()
                            .0
                            .send(RenderMessage::Resized)
                            .unwrap();
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let _ = wm.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height,
                        });
                    }
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        modifiers: _,
                    } => {
                        CHANNELS
                            .get()
                            .unwrap()
                            .0
                            .send(RenderMessage::MouseMove(position.x, position.y))
                            .unwrap();
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        modifiers,
                    } => {
                        CHANNELS
                            .get()
                            .unwrap()
                            .0
                            .send(RenderMessage::MouseState(*state, *button))
                            .unwrap();
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        CHANNELS
                            .get()
                            .unwrap()
                            .0
                            .send(RenderMessage::CharTyped(*c, current_modifiers.bits()))
                            .unwrap();
                    }
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    } => {
                        // input.scancode
                        match input.virtual_keycode {
                            None => {}
                            Some(keycode) => CHANNELS
                                .get()
                                .unwrap()
                                .0
                                .send(RenderMessage::KeyState(
                                    keycode as u32,
                                    input.scancode,
                                    match input.state {
                                        ElementState::Pressed => 0,
                                        ElementState::Released => 1,
                                    },
                                    current_modifiers.bits(),
                                ))
                                .unwrap(),
                        }
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        current_modifiers = *new_modifiers;
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                wm.upload_camera();

                let mc_state = MC_STATE.get().unwrap().load();

                let mut pipelines = Vec::new();
                if mc_state.render_world {
                    wm.update_animated_textures((SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() / 50) as u32);
                    pipelines.push(&TerrainPipeline as &dyn WmPipeline);
                } else {
                    pipelines.push(GL_PIPELINE.get().unwrap());
                }

                let surface = wm.wgpu_state.surface.as_ref().unwrap();
                let texture = surface.get_current_texture().unwrap();
                let view = texture.texture.create_view(
                    &wgpu::TextureViewDescriptor {
                        label: None,
                        format: Some(wgpu::TextureFormat::Bgra8Unorm),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: Default::default(),
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None
                    }
                );

                wm.render(&pipelines, &view);

                texture.present();
                
            }
            _ => {}
        }
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_cacheBlockStates(
    env: JNIEnv,
    class: JClass,
) {
    let wm = RENDERER.get().unwrap();

    {
        let blocks = BLOCKS.get().unwrap().lock();

        let blockstates = blocks.iter().map(|identifier| {
             (identifier.clone(), ResourcePath::try_from(&identifier[..]).unwrap().prepend("blockstates/").append(".json"))
        }).collect::<Vec<_>>();

        wm.mc.bake_blocks(wm, blockstates.iter().map(|(string, resource)| (string, resource)));
    }

    {
        let block_manager = wm.mc.block_manager.read();

        let default_missing_block = block_manager.blocks.get_full("minecraft:bedrock").unwrap().0 as u16;

        JAVA_BLOCK_STATES
            .get()
            .unwrap()
            .read()
            .iter()
            .for_each(|(key, global_ref)| {
                let offset = block_manager
                    .blocks
                    .get_full(key)
                    .map(|(index, _, _)| index as u16)
                    .unwrap_or(default_missing_block);

                if offset != 27 {
                    println!("{} {}", key, offset);
                }

                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "helperSetBlockStateIndex",
                    "(Ljava/lang/Object;J)V",
                    &[
                        JValue::Object(global_ref.as_obj()),
                        JValue::Long(offset as jlong),
                    ],
                )
                .unwrap();
            });
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_runHelperThread(
    env: JNIEnv,
    class: JClass,
) {
    let (_, rx) = CHANNELS.get_or_init(|| {
        let (tx, rx) = unbounded();
        (tx, rx)
    });

    //Wait until wgpu-mc is initialized
    while RENDERER.get().is_none() {}

    for render_message in rx.iter() {
        match render_message {
            RenderMessage::SetTitle(title) => WINDOW.get().unwrap().set_title(&title),
            RenderMessage::Task(func) => func(),
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
            RenderMessage::Resized => {
                env.call_static_method("dev/birb/wgpu/render/Wgpu", "onResize", "()V", &[])
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
        };
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_preInit(_env: JNIEnv, _class: JClass) {
    gl::init();

    MOUSE_STATE.set(Arc::new(ArcSwap::new(Arc::new(MouseState {
        x: 0.0,
        y: 0.0,
    }))));
    GL_PIPELINE.set(GlPipeline {
        commands: ArcSwap::new(Arc::new(Vec::new())),
        blank_texture: OnceCell::new(),
    });
    CHANNELS.get_or_init(|| {
        let (tx, rx) = unbounded();
        (tx, rx)
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_digestInputStream(
    env: JNIEnv,
    _class: JClass,
    input_stream: JObject,
) -> jbyteArray {
    let mut vec = Vec::with_capacity(1024);
    let array = env.new_byte_array(1024).unwrap();

    loop {
        let bytes_read = env
            .call_method(input_stream, "read", "([B)I", &[array.into()])
            .unwrap()
            .i()
            .unwrap();

        //bytes_read being -1 means EOF
        if bytes_read > 0 {
            let elements = env
                .get_byte_array_elements(array, ReleaseMode::NoCopyBack)
                .unwrap();

            let slice: &[u8] = unsafe {
                std::mem::transmute(std::slice::from_raw_parts(
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

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_updateWindowTitle(
    env: JNIEnv,
    _class: JClass,
    jtitle: JString,
) {
    let (tx, _) = CHANNELS.get().unwrap();

    let title: String = env.get_string(jtitle).unwrap().into();

    tx.send(RenderMessage::SetTitle(title));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_bakeBlockModels(
    env: JNIEnv,
    _class: JClass,
) -> jobject {
    let renderer = RENDERER.get().unwrap();

    let block_hashmap = env.new_object("java/util/HashMap", "()V", &[]).unwrap();

    // let instant = Instant::now();
    // renderer.mc.block_manager.read().baked_block_variants.iter().for_each(|(identifier, (key, _))| {
    //     let _integer = env.new_object("java/lang/Integer", "(I)V", &[
    //         JValue::Int(*key as i32)
    //     ]).unwrap();
    //
    //     env.call_method(block_hashmap, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[
    //         JValue::Object(env.new_string(identifier.to_string()).unwrap().into()),
    //         JValue::Object(_integer)
    //     ]).unwrap();
    // });
    // println!("Uploaded blocks to java HashMap in {}ms", Instant::now().duration_since(instant).as_millis());

    block_hashmap.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setWorldRenderState(
    _env: JNIEnv,
    _class: JClass,
    boolean: jboolean,
) {
    MC_STATE
        .get()
        .unwrap()
        .store(Arc::new(MinecraftRenderState {
            render_world: boolean != 0,
        }));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_submitCommands(
    _env: JNIEnv,
    _class: JClass,
) {
    let mut commands = gl::GL_COMMANDS.get().unwrap().clone().write();

    GL_PIPELINE
        .get()
        .unwrap()
        .commands
        .store(Arc::new(commands.clone()));

    commands.clear();
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_texImage2D(
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
        _ => panic!("Unknown format {:x}", format),
    };

    //For when the renderer is initialized
    let task = move || {
        let area = width * height;
        //In bytes
        assert_eq!(_type, 0x1401);
        let size = area as usize * 4;

        let data = if pixels_ptr != 0 {
            Vec::from(unsafe { std::slice::from_raw_parts(pixels_ptr as *const u8, size) })
        } else {
            vec![0; size]
        };

        let wm = RENDERER.get().unwrap();

        let tsv = TextureSamplerView::from_rgb_bytes(
            &wm.wgpu_state,
            &data[..],
            wgpu::Extent3d {
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
            BindableTexture::from_tsv(&wm.wgpu_state, &wm.render_pipeline_manager.load(), tsv);

        {
            println!("GL ALLOC {}", gl::GL_ALLOC.get().unwrap().read().len());
        }

        {
            gl::GL_ALLOC.get().unwrap().write().insert(
                texture_id,
                GlTexture {
                    width: width as u16,
                    height: height as u16,
                    bindable_texture: Some(Arc::new(bindable)),
                    pixels: data,
                },
            );
        }
    };

    let (tx, _) = CHANNELS.get_or_init(|| {
        let (tx, rx) = unbounded();
        (tx, rx)
    });

    tx.send(RenderMessage::Task(Box::new(task)));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_subImage2D(
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
        _ => panic!("Unknown format {:x}", format),
    };

    //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glPixelStore.xhtml
    let row_width = if unpack_row_length > 0 {
        unpack_row_length as i64
    } else {
        width as i64
    };

    let src_row_size = row_width as usize * pixel_size as usize;

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
        Vec::from(std::slice::from_raw_parts(
            pixels as *mut u8,
            next_row_byte_offset * height,
        ))
    };

    //For when the renderer is initialized
    let task = move || {
        let wm = RENDERER.get().unwrap();

        let gl_alloc = gl::GL_ALLOC.get().unwrap();
        let mut alloc_write = gl_alloc.write();

        let gl_texture = alloc_write.get_mut(&texture_id).unwrap();

        let dest_row_size = gl_texture.width as usize * pixel_size as usize;

        let mut pixel_offset = 0usize;
        for y in 0..height {
            let src_row_slice = &vec[pixel_offset..pixel_offset + src_row_size];
            pixel_offset += next_row_byte_offset;

            let dest_begin =
                (dest_row_size * (y + offsetY as usize)) + (offsetX as usize * pixel_size);
            let dest_end = dest_begin + src_row_size;

            let dest_row_slice = &mut gl_texture.pixels[dest_begin as usize..dest_end as usize];
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

    let (tx, _) = CHANNELS.get_or_init(|| {
        let (tx, rx) = unbounded();
        (tx, rx)
    });

    tx.send(RenderMessage::Task(Box::new(task)));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getMaxTextureSize(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let wm = RENDERER.get().unwrap();
    wm.wgpu_state.adapter.limits().max_texture_dimension_2d as i32
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getWindowWidth(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    RENDERER
        .get()
        .map_or(1280, |wm| wm.wgpu_state.surface_config.as_ref().unwrap().load().width as i32)
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getWindowHeight(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    RENDERER
        .get()
        .map_or(720, |wm| wm.wgpu_state.surface_config.as_ref().unwrap().load().height as i32)
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_clearColor(
    _env: JNIEnv,
    _class: JClass,
    r: jfloat,
    g: jfloat,
    b: jfloat,
) {
    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::ClearColor(r, g, b));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_attachTextureBindGroup(
    _env: JNIEnv,
    _class: JClass,
    slot: jint,
    id: jint,
) {
    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::AttachTexture(slot, id));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_wmUsePipeline(
    _env: JNIEnv,
    _class: JClass,
    pipeline: jint,
) {
    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::UsePipeline(pipeline as usize));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getVideoMode(
    env: JNIEnv,
    class: JClass,
) -> jstring {
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
        video_mode.refresh_rate(),
        video_mode.bit_depth()
    ))
    .unwrap()
    .into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createArrayPaletteStore(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    0
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setProjectionMatrix(
    env: JNIEnv,
    _class: JClass,
    float_array: jfloatArray,
) {
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
        use byteorder::ByteOrder;
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    let slice_4x4: [[f32; 4]; 4] = *bytemuck::from_bytes(bytemuck::cast_slice(&converted));

    let matrix = Matrix4::from(slice_4x4) * Matrix4::from_nonuniform_scale(1.0, 1.0, 0.0);

    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::SetMatrix(matrix));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_drawIndexed(
    _env: JNIEnv,
    _class: JClass,
    count: jint,
) {
    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::DrawIndexed(count as u32));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setVertexBuffer(
    env: JNIEnv,
    _class: JClass,
    byte_array: jbyteArray,
) {
    let mut bytes = vec![0; env.get_array_length(byte_array).unwrap() as usize];
    env.get_byte_array_region(byte_array, 0, &mut bytes[..])
        .unwrap();

    let byte_slice = bytemuck::cast_slice(&bytes);
    let mut cursor = Cursor::new(byte_slice);
    let mut converted = Vec::with_capacity(bytes.len() / 4);

    assert_eq!(bytes.len() % 4, 0);

    for _ in 0..bytes.len() / 4 {
        use byteorder::ByteOrder;
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::SetVertexBuffer(Vec::from(bytemuck::cast_slice(
            &converted,
        ))));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setIndexBuffer(
    env: JNIEnv,
    _class: JClass,
    int_array: jintArray,
) {
    let elements = env
        .get_int_array_elements(int_array, ReleaseMode::NoCopyBack)
        .unwrap();

    let slice = unsafe {
        slice::from_raw_parts(
            elements.as_ptr() as *mut u32,
            elements.size().unwrap() as usize,
        )
    };

    gl::GL_COMMANDS
        .get()
        .unwrap()
        .write()
        .push(GLCommand::SetIndexBuffer(Vec::from(slice)));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_scheduleChunkRebuild(
    env: JNIEnv,
    class: JClass,
    x: jint,
    z: jint,
) {
    let wm = RENDERER.get().unwrap();

    // println!("Building chunk {},{}", x, z);
    // wm.mc.chunks.loaded_chunks.read().get(&(x,z)).unwrap().load().bake(&wm.mc.block_manager.read());
    // wm.mc.chunks.assemble_world_meshes(wm);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createPalette(
    env: JNIEnv,
    class: JClass,
    idList: jlong,
) -> jlong {
    let mut palette = Box::new(JavaPalette::new(idList));

    let ptr = ((&mut *palette as *mut JavaPalette) as usize) as jlong;
    mem::forget(palette);

    ptr
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_clearPalette(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
) {
    let palette = (palette_long as usize) as *mut JavaPalette;

    unsafe { palette.as_mut().unwrap().clear() };
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_destroyPalette(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
) {
    let palette = (palette_long as usize) as *mut JavaPalette;

    unsafe { Box::from_raw(palette) };
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteIndex(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
    object: JObject,
    blockstate_index: jint,
) -> jint {
    let mut palette = (palette_long as usize) as *mut JavaPalette;

    (unsafe {
        palette.as_mut().unwrap().index((
            env.new_global_ref(object).unwrap(),
            (blockstate_index as u32).into(),
        ))
    }) as jint
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteHasAny(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
    predicate: JObject,
) -> jint {
    let palette = (palette_long as usize) as *mut JavaPalette;

    //horribly slow. how nice
    (unsafe {
        palette
            .as_ref()
            .unwrap()
            .has_any(&*Box::new(|object: jobject| {
                env.call_method(
                    predicate,
                    "test",
                    "(Ljava/lang/Object;)Z",
                    &[JValue::Object(JObject::from(object))],
                )
                .unwrap()
                .z()
                .unwrap()
            }))
    }) as jint
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteSize(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
) -> jint {
    let palette = (palette_long as usize) as *mut JavaPalette;

    (unsafe { palette.as_ref().unwrap().size() }) as jint
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_copyPalette(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
) -> jlong {
    let palette = (palette_long as usize) as *mut JavaPalette;
    let mut new_palette = Box::new(unsafe { palette.as_ref().unwrap().clone() });
    let new_palette_ptr = &mut *new_palette as *mut JavaPalette;
    std::mem::forget(new_palette);

    new_palette_ptr as usize as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteGet(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
    index: i32,
) -> jobject {
    let palette = (palette_long as usize) as *mut JavaPalette;

    (unsafe { palette.as_ref().unwrap().get(index as usize).unwrap() })
        .0
        .as_obj()
        .into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_paletteReadPacket(
    env: JNIEnv,
    class: JClass,
    palette_long: jlong,
    array: jbyteArray,
    current_position: jint,
    blockstate_offsets: jlongArray,
) -> jint {
    let palette = unsafe {
        ((palette_long as usize) as *mut JavaPalette)
            .as_mut()
            .unwrap()
    };
    let array = env
        .get_byte_array_elements(array, ReleaseMode::NoCopyBack)
        .unwrap();

    let blockstate_offsets_array = env
        .get_int_array_elements(blockstate_offsets, ReleaseMode::NoCopyBack)
        .unwrap();

    let id_list = unsafe { palette.id_list.as_ref().unwrap() };

    let blockstate_offsets = unsafe {
        std::slice::from_raw_parts(
            blockstate_offsets_array.as_ptr() as *mut i32,
            blockstate_offsets_array.size().unwrap() as usize,
        )
    };

    let vec = unsafe {
        std::slice::from_raw_parts(
            array.as_ptr().offset(current_position as isize) as *mut u8,
            (array.size().unwrap() - current_position) as usize,
        )
    };

    let mut cursor = Cursor::new(vec);
    let packet_len: i32 = cursor.read_var_int().unwrap().into();

    for i in 0..packet_len as usize {
        let var_int: i32 = cursor.read_var_int().unwrap().into();

        let object = id_list.map.get(&var_int.into()).unwrap().clone();

        palette.add((object, BlockstateKey {
            block: (blockstate_offsets[i] >> 16) as u16,
            augment: (blockstate_offsets[i] & 0xffff) as u16
        }));
    }

    //The amount of bytes read
    cursor.position() as jint
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createIdList(
    env: JNIEnv,
    class: JClass,
) -> jlong {
    let mut palette = Box::new(IdList::new());

    let ptr = ((&mut *palette as *mut IdList) as usize) as jlong;
    mem::forget(palette);

    ptr
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_addIdListEntry(
    env: JNIEnv,
    class: JClass,
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

#[derive(Debug)]
pub struct PackedIntegerArray {
    data: Box<[i64]>,
    elements_per_long: i32,
    element_bits: i32,
    max_value: i64,
    index_scale: i32,
    index_offset: i32,
    index_shift: i32,
    size: i32
}

impl PackedIntegerArray {
    pub fn get(&self, x: i32, y: u16, z: i32) -> i32 {
        let x = x & 0xf;
        let y = y & 0xf;
        let z = z & 0xf;

        self.get_by_index(((((y as i32) << 4) | z) << 4) | x)
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
    class: JClass,
    data: jlongArray,
    elements_per_long: jint,
    element_bits: jint,
    max_value: jlong,
    index_scale: jint,
    index_offset: jint,
    index_shift: jint,
    size: jint,
) -> jlong {
    let copy = env.get_long_array_elements(data, ReleaseMode::NoCopyBack)
        .unwrap();

    let packed_arr = Box::new(PackedIntegerArray {
        data: Vec::from(
            unsafe {
                std::slice::from_raw_parts(copy.as_ptr(), copy.size().unwrap() as usize)
            }
        ).into_boxed_slice(),
        elements_per_long,
        element_bits,
        max_value,
        index_scale,
        index_offset,
        index_shift,
        size,
    });

    let ptr = (&*packed_arr as *const PackedIntegerArray) as usize;

    std::mem::forget(packed_arr);

    ptr as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_debugPalette(
    env: JNIEnv,
    class: JClass,
    packed_integer_array: jlong,
    index: jint
) {
    let array = unsafe { ((packed_integer_array as usize) as *mut PackedIntegerArray).as_ref().unwrap() };
    println!(
        "array val index: {} computed: {} ptr: {} raw read: {} val: {}\n{:?}",
        index,
        array.compute_storage_index(index),
        array.debug_pointer(index),
        unsafe { (array.debug_pointer(index) as *mut i64).read_volatile() },
        array.get_by_index(index),
        array
    );
    // dbg!(array.index_offset, array.index_scale, array.index_shift, array.element_bits, array.size, array.element_bits, array.elements_per_long);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getRawStoragePointer(
    env: JNIEnv,
    class: JClass,
    packed_integer_array: jlong
) -> jlong {
    unsafe { ((packed_integer_array as usize) as *mut PackedIntegerArray).as_ref().unwrap().data.as_ptr() as jlong }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_destroyPaletteStorage(
    env: JNIEnv,
    class: JClass,
    storage: jlong,
) {
    unsafe {
        Box::from_raw((storage as usize) as *mut PackedIntegerArray);
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setCursorPosition(
    env: JNIEnv,
    class: JClass,
    x: f64,
    y: f64,
) {
    WINDOW
        .get()
        .unwrap()
        .set_cursor_position(PhysicalPosition { x, y });
}

const GLFW_CURSOR_NORMAL: i32 = 212993;
const GLFW_CURSOR_HIDDEN: i32 = 212994;
const GLFW_CURSOR_DISABLED: i32 = 212995;
/// See https://www.glfw.org/docs/3.3/input_guide.html#cursor_mode
#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setCursorMode(
    env: JNIEnv,
    class: JClass,
    mode: i32,
) {
    match mode {
        GLFW_CURSOR_NORMAL => {
            WINDOW.get().unwrap().set_cursor_grab(false).unwrap();
            WINDOW.get().unwrap().set_cursor_visible(true);
        }
        GLFW_CURSOR_HIDDEN => {
            WINDOW.get().unwrap().set_cursor_grab(false).unwrap();
            WINDOW.get().unwrap().set_cursor_visible(false);
        }
        GLFW_CURSOR_DISABLED => {
            WINDOW.get().unwrap().set_cursor_grab(true).unwrap();
            WINDOW.get().unwrap().set_cursor_visible(false);
        }
        _ => {
            println!("Set cursor mode had an invalid mode.")
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setCamera(
    env: JNIEnv,
    class: JClass,
    x: jdouble,
    y: jdouble,
    z: jdouble,
    yaw: jfloat,
    pitch: jfloat
) {
    let renderer = RENDERER.get().unwrap();
    if renderer.mc.camera_bind_group.load().is_none() {
        renderer.mc.init_camera(renderer);
    }

    let mut camera = **renderer.mc.camera.load();
    camera.position = Point3::new(x as f32, y as f32, z as f32);
    camera.yaw = (PI / 180.0) * yaw;
    camera.pitch = (PI / 180.0) * pitch;

    renderer.mc.camera.store(Arc::new(camera));
    renderer.upload_camera();
}
