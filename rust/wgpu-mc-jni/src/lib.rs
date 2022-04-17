#![feature(once_cell)]
#![feature(mixed_integer_ops)]

extern crate core;

use core::slice;
use std::{thread};
use std::convert::{TryFrom};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{channel};
use std::time::Instant;
use arc_swap::{ArcSwap};
use cgmath::{Matrix4, Vector3};
use futures::executor::block_on;
use jni::{JavaVM, JNIEnv};
use jni::objects::{JClass, JObject, JString, JValue, ReleaseMode};
use jni::sys::{jboolean, jbyteArray, jint, jintArray, jobject, jstring, jlong, jfloat, jfloatArray, jdouble, _jobject};
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::Window;
use gl::pipeline::{GLCommand, GlPipeline};
use wgpu_mc::{HasWindowSize, WindowSize, WmRenderer};
use wgpu_mc::mc::block::{Block};
use wgpu_mc::mc::datapack::{NamespacedResource};
use wgpu_mc::mc::resource::ResourceProvider;
use wgpu_mc::model::BindableTexture;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::texture::TextureSamplerView;
use wgpu_mc::wgpu;
use std::io::Cursor;
use std::num::NonZeroU32;
use wgpu::{Extent3d};
use crate::gl::{GlTexture};
use byteorder::{LittleEndian, ReadBytesExt};
use once_cell::sync::OnceCell;
use wgpu_mc::render::pipeline::terrain::TerrainPipeline;
use wgpu_mc::wgpu::ImageDataLayout;

mod mc_interface;
mod gl;
mod palette;

enum RenderMessage {
    SetTitle(String),
    Task(Box<dyn FnOnce() + Send + Sync>),
    KeyPressed(u32),
    MouseState(ElementState, MouseButton),
    MouseMove(f64, f64),
    Resized
}

struct MinecraftRenderState {
    //draw_queue: Vec<>,
    render_world: bool
}

struct MouseState {
    pub x: f64,
    pub y: f64
}

static RENDERER: OnceCell<WmRenderer> = OnceCell::new();
static CHANNELS: OnceCell<(mpsc::SyncSender<RenderMessage>, Mutex<mpsc::Receiver<RenderMessage>>)> = OnceCell::new();
static MC_STATE: OnceCell<RwLock<MinecraftRenderState>> = OnceCell::new();
static MOUSE_STATE: OnceCell<Arc<ArcSwap<MouseState>>> = OnceCell::new();
static GL_PIPELINE: OnceCell<GlPipeline> = OnceCell::new();
static WINDOW: OnceCell<Arc<Window>> = OnceCell::new();

struct WinitWindowWrapper<'a> {
    window: &'a Window
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

struct JarShaderProvider {

}

struct MinecraftResourceManagerAdapter {
    jvm: JavaVM
}

impl ResourceProvider for MinecraftResourceManagerAdapter {
    fn get_resource(&self, id: &NamespacedResource) -> Vec<u8> {
        let env = self.jvm.attach_current_thread()
            .unwrap();

        let ident_ns = env.new_string(&id.0).unwrap();
        let ident_path = env.new_string(&id.1).unwrap();

        let bytes = env.call_static_method(
            "dev/birb/wgpu/rust/WgpuResourceProvider",
            "getResource",
            "(Ljava/lang/String;Ljava/lang/String;)[B",
            &[
                JValue::Object(ident_ns.into()),
                JValue::Object(ident_path.into())
            ]).unwrap().l().unwrap();

        let elements = env.get_byte_array_elements(bytes.into_inner(), ReleaseMode::NoCopyBack)
            .unwrap();

        let mut vec = vec![0u8; elements.size().unwrap() as usize];
        vec.copy_from_slice(
            unsafe {
                std::slice::from_raw_parts(elements.as_ptr() as *const u8, elements.size().unwrap() as usize)
            }
        );

        vec
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_uploadChunk(
    env: JNIEnv,
    _class: JClass,
    world_chunk: JObject) {

    mc_interface::chunk_from_java_world_chunk(&env, &world_chunk);

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getBackend(
    env: JNIEnv,
    _class: JClass) -> jstring {

    let renderer = RENDERER.get().unwrap();
    let backend = renderer.get_backend_description();

    env.new_string(backend)
         .unwrap()
        .into_inner()

}

// #[no_mangle]
// pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_registerSprite(
//     env: JNIEnv,
//     class: JClass,
//     sprite_type: JString,
//     jnamespace: JString,
//     buffer: jbyteArray) {
//
//     let renderer = unsafe { RENDERER.get().unwrap() };
//
//
//
//     let namespace_str: String = env.get_string(jnamespace).unwrap().into();
//     let sprite_type: String = env.get_string(sprite_type).unwrap().into();
//     let identifier = NamespacedResource::try_from(namespace_str.as_str())
//         .unwrap();
//
//     let buffer_arr = env.get_byte_array_elements(
//         buffer,
//         ReleaseMode::NoCopyBack)
//         .unwrap();
//
//     let buffer_arr_size = buffer_arr.size().unwrap() as usize;
//     let mut buffer: Vec<u8> = vec![0; buffer_arr_size];
//     //Create a slice looking into the buffer so that we can immediately copy the data into a Vec.
//     let slice: &[u8] = unsafe {
//         std::mem::transmute(
//             std::slice::from_raw_parts(
//                 buffer_arr.as_ptr(),
//                 buffer_arr_size)
//         )
//     };
//
//     buffer.copy_from_slice(slice);
//
//     // println!("{:?}", renderer.mc.texture_manager.textures);
//     renderer.mc.texture_manager.textures.insert(identifier, buffer);
// }

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_registerEntry(
    env: JNIEnv,
    _class: JClass,
    entry_type: jint,
    name: JString) {

    let rname: String = env.get_string(name).unwrap().into();

    let renderer = RENDERER.get().unwrap();

    let mut block_manager = renderer.mc.block_manager.write();

    match entry_type {
        0 => {
            let identifier = NamespacedResource::try_from(&rname[..]).unwrap();

            let resource = renderer.mc.resource_provider.get_resource(
                &identifier.prepend("blockstates/").append(".json")
            );
            let json_str = std::str::from_utf8(&resource).unwrap();
            let model = Block::from_json(
                &identifier.to_string(),
                json_str
            ).or_else(|| {
                Block::from_json(
                    &identifier.to_string(),
                    "{\"variants\":{\"\": {\"model\": \"minecraft:block/bedrock\"}}}"
                )
            }).unwrap();
            block_manager.blocks.insert(identifier, model);
        },
        _ => unimplemented!()
    };

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_startRendering(
    env: JNIEnv,
    _class: JClass,
    string: JString
) {

    use winit::event_loop::EventLoop;

    let title: String = env.get_string(string).unwrap().into();

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
                height: 720
            }))
            .build(&event_loop)
            .unwrap()
    );

    WINDOW.set(window.clone());

    MC_STATE.set(RwLock::new(MinecraftRenderState {
        render_world: false
    }));

    let wrapper = &WinitWindowWrapper {
        window: &window
    };

    println!("[wgpu-mc] initializing wgpu");

    let wgpu_state = block_on(
        WmRenderer::init_wgpu(wrapper)
    );

    let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
        jvm: env.get_java_vm().unwrap()
    });

    println!("[wgpu-mc] initializing");

    let wm = WmRenderer::new(
        wgpu_state,
        resource_provider
    );

    wm.init(
        &[
            &TerrainPipeline,
            GL_PIPELINE.get().unwrap()
        ]
    );

    RENDERER.set(wm.clone());

    println!("[wgpu-mc] done initializing");

    env.set_static_field(
        "dev/birb/wgpu/render/Wgpu",
        (
            "dev/birb/wgpu/render/Wgpu",
            "INITIALIZED",
            "Z"
        ),
        JValue::Bool(true.into())
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let _mc_state = MC_STATE.get().unwrap().read();

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
                            height: physical_size.height
                        });
                        CHANNELS.get().unwrap().0.send(RenderMessage::Resized).unwrap();
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let _ = wm.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height
                        });
                    }
                    WindowEvent::CursorMoved {
                        device_id: _, position, modifiers: _
                    } => {
                        CHANNELS.get().unwrap().0.send(RenderMessage::MouseMove(
                            position.x,
                            position.y
                        )).unwrap();
                    },
                    WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                        CHANNELS.get().unwrap().0.send(RenderMessage::MouseState(
                            *state, *button
                        )).unwrap();
                    },
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                wm.update();

                wm.render(&[
                    GL_PIPELINE.get().unwrap()
                ]);
            }
            _ => {}
        }
    });

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_runHelperThread(
    env: JNIEnv,
    class: JClass
) {
    let (_, rx) = CHANNELS.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<RenderMessage>(1024);
        (tx, Mutex::new(rx))
    });
    let rx = rx.lock();

    //Wait until wgpu-mc is initialized
    while RENDERER.get().is_none() {};

    for render_message in rx.iter() {
        match render_message {
            // RenderMessage::SetTitle(title) => WINDOW.get().unwrap().set_title(&title),
            RenderMessage::Task(func) => func(),
            RenderMessage::KeyPressed(_) => {}
            RenderMessage::MouseMove(x, y) => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "mouseMove",
                    "(DD)V",
                    &[
                        JValue::Double(x),
                        JValue::Double(y)
                    ]
                ).unwrap();
            }
            RenderMessage::MouseState(element_state, mouse_button) => {
                let button = match mouse_button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Other(_) => 0
                };

                let action = match element_state {
                    ElementState::Pressed => 1,
                    ElementState::Released => 0
                };

                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "mouseAction",
                    "(II)V",
                    &[
                        JValue::Int(button),
                        JValue::Int(action)
                    ]
                ).unwrap();
            },
            RenderMessage::Resized => {
                env.call_static_method(
                    "dev/birb/wgpu/render/Wgpu",
                    "onResize",
                    "()V",
                    &[]
                ).unwrap();
            },
            _ => {}
        };
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_preInit(
    _env: JNIEnv,
    _class: JClass
) {
    gl::init();

    MOUSE_STATE.set(Arc::new(ArcSwap::new(
        Arc::new(MouseState {
            x: 0.0,
            y: 0.0
        })
    )));
    GL_PIPELINE.set(GlPipeline {
        commands: ArcSwap::new(Arc::new(Vec::new())),
        black_texture: OnceCell::new()
    });
    CHANNELS.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<RenderMessage>(1024);
        (tx, Mutex::new(rx))
    });

    println!("wgpu-mc pre initialized");
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_digestInputStream(
    env: JNIEnv,
    _class: JClass,
    input_stream: JObject) -> jbyteArray {

    let mut vec = Vec::with_capacity(1024);
    let array = env.new_byte_array(1024).unwrap();

    loop {
        let bytes_read = env.call_method(
            input_stream,
            "read",
            "([B)I",
            &[array.into()]).unwrap().i().unwrap();

        //bytes_read being -1 means EOF
        if bytes_read > 0 {
            let elements = env.get_byte_array_elements(array, ReleaseMode::NoCopyBack)
                .unwrap();

            let slice: &[u8] = unsafe {
                std::mem::transmute(
                    std::slice::from_raw_parts(elements.as_ptr(), bytes_read as usize)
                )
            };

            vec.extend_from_slice(slice);
        } else {
            break;
        }
    }


    let bytes = env.new_byte_array(vec.len() as i32).unwrap();
    let bytes_elements = env.get_byte_array_elements(bytes, ReleaseMode::CopyBack).unwrap();

    unsafe {
        std::ptr::copy(vec.as_ptr(), bytes_elements.as_ptr() as *mut u8, vec.len());
    }

    bytes

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_updateWindowTitle(
    env: JNIEnv,
    _class: JClass,
    jtitle: JString) {

    let (tx, _) = CHANNELS.get().unwrap();

    let title: String = env.get_string(jtitle).unwrap().into();

    tx.send(RenderMessage::SetTitle(title));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_bakeBlockModels(
    env: JNIEnv,
    _class: JClass) -> jobject {

    let renderer = RENDERER.get().unwrap();

    let block_hashmap = env.new_object("java/util/HashMap", "()V", &[])
        .unwrap();

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
    boolean: jboolean) {

    let render_state = MC_STATE.get().unwrap();
    render_state.write().render_world = boolean != 0;
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_submitCommands(
    _env: JNIEnv,
    _class: JClass
) {
    let mut commands = gl::GL_COMMANDS.get().unwrap().clone().write();

    GL_PIPELINE.get().unwrap().commands.store(
        Arc::new(
            commands.clone()
        )
    );

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
    pixels_ptr: jlong
) {
    let _pixel_size = match format {
        0x1908 | 0x80E1 => 4,
        _ => panic!("Unknown format {:x}", format)
    };

    //For when the renderer is initialized
    let task = move || {
        let area = width * height;
        //In bytes
        assert_eq!(_type, 0x1401);
        let size = area as usize * 4;

        let data = if pixels_ptr != 0 {
            Vec::from(
                unsafe {
                    std::slice::from_raw_parts(pixels_ptr as *const u8, size)
                }
            )
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
                depth_or_array_layers: 1
            },
            None,
            match format {
                0x1908 => wgpu::TextureFormat::Rgba8Unorm,
                0x80E1 => wgpu::TextureFormat::Bgra8Unorm,
                _ => unimplemented!()
            }
        ).unwrap();

        let bindable = BindableTexture::from_tsv(
            &wm.wgpu_state,
            &wm.render_pipeline_manager.load(),
            tsv,
        );

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
                    pixels: data
                },
            );
        }
    };

    let (tx, _) = CHANNELS.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<RenderMessage>(1024);
        (tx, Mutex::new(rx))
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
    unpack_alignment: jint
) {
    let mut pixels = pixels as usize;
    let unpack_row_length = unpack_row_length as usize;
    let unpack_skip_pixels = unpack_skip_pixels as usize;
    let unpack_skip_rows = unpack_skip_rows as usize;
    let unpack_alignment = unpack_alignment as usize;
    let width = width as usize;
    let height = height as usize;

    //For when the renderer is initialized
    let task = move || {
        let wm = RENDERER.get().unwrap();

        let gl_alloc = gl::GL_ALLOC.get().unwrap();
        let mut alloc_write = gl_alloc.write();

        let gl_texture = alloc_write.get_mut(&texture_id).unwrap();

        let pixel_size = match format {
            0x1908 | 0x80E1 => 4,
            _ => panic!("Unknown format {:x}", format)
        };

        //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glPixelStore.xhtml
        let row_width = if unpack_row_length > 0 {
            unpack_row_length as i64
        } else {
            width as i64
        };

        let src_row_size = row_width as usize * pixel_size as usize;
        let dest_row_size = gl_texture.width as usize * pixel_size as usize;

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

        let mut pixel_offset = 0usize;
        for y in 0..height {
            let src_row_start_ptr = unsafe { (pixels + pixel_offset) as *const u8 };
            let src_row_slice = unsafe { std::slice::from_raw_parts(src_row_start_ptr, src_row_size) };
            pixel_offset += next_row_byte_offset;

            let dest_begin = (dest_row_size * (y + offsetY as usize)) + (offsetX as usize * pixel_size);
            let dest_end = dest_begin + src_row_size;

            let dest_row_slice = &mut gl_texture.pixels[dest_begin as usize..dest_end as usize];
            dest_row_slice.copy_from_slice(src_row_slice);
        }

        wm.wgpu_state.queue.write_texture(
            gl_texture.bindable_texture.as_ref().unwrap().tsv.texture.as_image_copy(),
            &gl_texture.pixels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(gl_texture.width as u32 * 4),
                rows_per_image: NonZeroU32::new(gl_texture.height as u32)
            },
            Extent3d {
                width: gl_texture.width as u32,
                height: gl_texture.height as u32,
                depth_or_array_layers: 1
            }
        );
    };

    let (tx, _) = CHANNELS.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<RenderMessage>(1024);
        ( tx, Mutex::new(rx))
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
    RENDERER.get().unwrap().wgpu_state.surface_config.load().width as i32
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getWindowHeight(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    RENDERER.get().unwrap().wgpu_state.surface_config.load().height as i32
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_clearColor(
    _env: JNIEnv,
    _class: JClass,
    r: jfloat,
    g: jfloat,
    b: jfloat
) {
    gl::GL_COMMANDS.get().unwrap().write().push(GLCommand::ClearColor(r, g, b));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_attachTextureBindGroup(
    _env: JNIEnv,
    _class: JClass,
    slot: jint,
    id: jint
) {
    gl::GL_COMMANDS.get().unwrap().write().push(GLCommand::AttachTexture(slot, id));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_wmUsePipeline(
    _env: JNIEnv,
    _class: JClass,
    pipeline: jint,
) {
    gl::GL_COMMANDS.get().unwrap().write().push(GLCommand::UsePipeline(pipeline as usize));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getVideoMode(
    env: JNIEnv,
    class: JClass
) -> jstring {
    let video_mode = WINDOW.get().unwrap().current_monitor().unwrap().video_modes().find(|_| true).unwrap();
    env.new_string(format!("{}x{}@{}:{}", video_mode.size().width, video_mode.size().height, video_mode.refresh_rate(), video_mode.bit_depth())).unwrap().into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_createArrayPaletteStore(
    _env: JNIEnv,
    _class: JClass
) -> jlong {
    0
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setProjectionMatrix(
    env: JNIEnv,
    _class: JClass,
    float_array: jfloatArray
) {
    let elements = env.get_float_array_elements(float_array, ReleaseMode::NoCopyBack)
        .unwrap();

    let slice = unsafe {
        slice::from_raw_parts(elements.as_ptr() as *mut f32, elements.size().unwrap() as usize)
    };

    let mut cursor = Cursor::new(bytemuck::cast_slice::<f32, u8>(slice));
    let mut converted = Vec::with_capacity(slice.len());

    for _ in 0..slice.len() {
        use byteorder::ByteOrder;
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    let slice_4x4: [[f32; 4]; 4] = *bytemuck::from_bytes(
        bytemuck::cast_slice(&converted)
    );

    let matrix = Matrix4::from(slice_4x4) * Matrix4::from_nonuniform_scale(1.0, 1.0, 0.0);

    gl::GL_COMMANDS.get().unwrap().write().push(
        GLCommand::SetMatrix(matrix)
    );
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_drawIndexed(
    _env: JNIEnv,
    _class: JClass,
    count: jint
) {
    gl::GL_COMMANDS.get().unwrap().write().push(
        GLCommand::DrawIndexed(count as u32)
    );
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setVertexBuffer(
    env: JNIEnv,
    _class: JClass,
    byte_array: jbyteArray
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

    gl::GL_COMMANDS.get().unwrap().write().push(
        GLCommand::SetVertexBuffer(Vec::from(bytemuck::cast_slice(&converted)))
    );
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setIndexBuffer(
    env: JNIEnv,
    _class: JClass,
    int_array: jintArray
) {
    let elements = env.get_int_array_elements(int_array, ReleaseMode::NoCopyBack)
        .unwrap();

    let slice = unsafe {
        slice::from_raw_parts(elements.as_ptr() as *mut u32, elements.size().unwrap() as usize)
    };

    gl::GL_COMMANDS.get().unwrap().write().push(
        GLCommand::SetIndexBuffer(Vec::from(slice))
    );
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getMouseX(
    _env: JNIEnv,
    _class: JClass,
) -> jdouble {
    MOUSE_STATE.get().unwrap().load().x
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getMouseY(
    _env: JNIEnv,
    _class: JClass,
) -> jdouble {
    MOUSE_STATE.get().unwrap().load().y
}