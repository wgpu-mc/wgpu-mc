#![feature(once_cell)]

use std::{fs, thread};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::lazy::OnceCell;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{channel, RecvError, Sender};
use std::time::Instant;

use arc_swap::ArcSwap;
use cgmath::{Matrix4, SquareMatrix};
use dashmap::DashMap;
use futures::executor::block_on;
use jni::{JavaVM, JNIEnv};
use jni::errors::Error;
use jni::objects::{JClass, JObject, JString, JValue, ReleaseMode};
use jni::sys::{jboolean, jbyteArray, jint, jintArray, jobject, jobjectArray, jstring, jlong};
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use gl::pipeline::{GLCommand, GlPipeline};
use wgpu_mc::{HasWindowSize, WindowSize, WmRenderer};
use wgpu_mc::mc::block::{Block, BlockDirection, BlockState};
use wgpu_mc::mc::BlockEntry;
use wgpu_mc::mc::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_VOLUME};
use wgpu_mc::mc::datapack::{NamespacedResource, TagOrResource};
use wgpu_mc::mc::resource::ResourceProvider;
use wgpu_mc::model::Material;
use wgpu_mc::render::chunk::BakedChunk;
use wgpu_mc::render::pipeline::builtin::WorldPipeline;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::texture::WgpuTexture;

use crate::mc_interface::xyz_to_index;
use std::cell::{RefCell, Cell};
use wgpu::{RenderPipelineDescriptor, PipelineLayoutDescriptor, TextureDescriptor, Extent3d, ImageDataLayout, BindGroupDescriptor, BindGroupEntry, BindingResource, TextureViewDescriptor};
use crate::gl::GL_COMMANDS;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use std::num::NonZeroU32;
use std::rc::Rc;

///Foreword here,
/// the code is quite messy and will be cleaned up at some point.
/// I'm sure there are even some bad practices atm but the general goal is to get things working
/// relatively well, and then clean them up, while still keeping the prototyped code updateable and
/// managable. Feel free to make a pull request.

mod mc_interface;
mod gl;

#[derive(Debug)]
enum RenderMessage {
    SetTitle(String)
}

struct MinecraftRenderState {
    //draw_queue: Vec<>,
    render_world: bool
}

static mut RENDERER: MaybeUninit<WmRenderer> = MaybeUninit::uninit();

static mut EVENT_LOOP: MaybeUninit<EventLoop<()>> = MaybeUninit::uninit();
static mut WINDOW: MaybeUninit<Window> = MaybeUninit::uninit();

static mut CHANNEL_TX: MaybeUninit<Mutex<mpsc::Sender<RenderMessage>>> = MaybeUninit::uninit();
static mut CHANNEL_RX: MaybeUninit<Mutex<mpsc::Receiver<RenderMessage>>> = MaybeUninit::uninit();

static mut MC_STATE: MaybeUninit<RwLock<MinecraftRenderState>> = MaybeUninit::uninit();

static mut GL_PIPELINE: MaybeUninit<GlPipeline> = MaybeUninit::uninit();

struct WinitWindowWrapper<'a> {
    window: &'a Window
}

impl HasWindowSize for &WinitWindowWrapper<'_> {
    fn get_window_size(&self) -> WindowSize {
        WindowSize {
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
        }
    }
}

unsafe impl HasRawWindowHandle for &WinitWindowWrapper<'_> {

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

        //Could just simplify this to a formatted string
        let bytes = match env.call_static_method(
            "dev/birb/wgpu/rust/WgpuResourceProvider",
            "getResource",
            "(Ljava/lang/String;Ljava/lang/String;)[B",
            &[
                JValue::Object(ident_ns.into()),
                JValue::Object(ident_path.into())
            ]) {
            Ok(jvalue) => jvalue.l().unwrap(),
            Err(e) => panic!("{:?}\nID {}", e, id)
        };

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
    class: JClass,
    world_chunk: JObject) {

    mc_interface::chunk_from_java_world_chunk(&env, &world_chunk);

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getBackend(
    env: JNIEnv,
    class: JClass) -> jstring {

    let renderer = unsafe { RENDERER.assume_init_ref() };
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
//     let renderer = unsafe { RENDERER.assume_init_ref() };
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
    class: JClass,
    entry_type: jint,
    name: JString) {

    let rname: String = env.get_string(name).unwrap().into();

    let renderer = unsafe { RENDERER.assume_init_mut() };

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
        // 1 => renderer.registry.items.insert(rname),
        _ => unimplemented!()
    };

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_initialize(
    env: JNIEnv,
    class: JClass,
    string: JString
) {

    use winit::event_loop::EventLoop;

    let title: String = env.get_string(string).unwrap().into();

    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title(&format!("{}", title))
        .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
            width: 1280,
            height: 720
        }))
        .build(&event_loop)
        .unwrap();

    let (tx, rx) = mpsc::channel::<RenderMessage>();

    unsafe {
        GL_PIPELINE = MaybeUninit::new(GlPipeline {
            pipelines: Default::default(),
            matrix_stack: RefCell::new([Matrix4::identity(); 32]),
            matrix_offset: RefCell::new(0),
            commands: ArcSwap::new(Arc::new(Vec::new())),
            active_texture_slot: RefCell::new(0),
            slots: RefCell::new(HashMap::new()),
            vertex_attributes: RefCell::new(HashMap::new()),
            client_states: RefCell::new(Vec::new()),
            shaders: RefCell::new(None)
        });
        CHANNEL_TX = MaybeUninit::new(Mutex::new(tx));
        CHANNEL_RX = MaybeUninit::new(Mutex::new(rx));
        EVENT_LOOP = MaybeUninit::new(event_loop);
        WINDOW = MaybeUninit::new(window);
        MC_STATE = MaybeUninit::new(RwLock::new(MinecraftRenderState {
            render_world: false
        }));
    }
    println!("we aint even in the same method anymo");
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_doEventLoop(
    env: JNIEnv,
    class: JClass) {
    // let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let mut swap: MaybeUninit<EventLoop<()>> = MaybeUninit::uninit();

    std::mem::swap(&mut swap, unsafe { &mut EVENT_LOOP });

    let renderer = unsafe { RENDERER.assume_init_mut() };

    let event_loop = unsafe { swap.assume_init() };

    let rx = unsafe { CHANNEL_RX.assume_init_ref() }.lock();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let mc_state = unsafe { MC_STATE.assume_init_ref() }.read();

        let (mut state, window) = unsafe {
            (
                RENDERER.assume_init_mut(),
                WINDOW.assume_init_ref()
            )
        };

        let mut msg_r = rx.try_recv();
        while msg_r.is_ok() {
            let msg = msg_r.unwrap();

            match msg {
                RenderMessage::SetTitle(title) => window.set_title(&title),
            };

            msg_r = rx.try_recv();
        }

        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        &state.resize(WindowSize {
                            width: physical_size.width,
                            height: physical_size.height
                        });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        &state.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height
                        });
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();

                state.render(&[
                    unsafe { GL_PIPELINE.assume_init_ref() }
                ]);

                // let delta = Instant::now().duration_since(frame_begin).as_millis()+1; //+1 so we don't divide by zero
                // frame_begin = Instant::now();

                // println!("Frametime {}, FPS {}", delta, 1000/delta);
            }
            _ => {}
        }
    });

    // });
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_initRenderer(
    env: JNIEnv,
    class: JClass
) {
    let wrapper = &WinitWindowWrapper {
        window: unsafe { WINDOW.assume_init_ref() }
    };
    let mut state = block_on(WmRenderer::new(
        &wrapper, Arc::new(MinecraftResourceManagerAdapter {
            jvm: env.get_java_vm().unwrap()
        }))
    );
    unsafe {
        RENDERER = MaybeUninit::new(state);
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_digestInputStream(
    env: JNIEnv,
    class: JClass,
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
    class: JClass,
    jtitle: JString) {

    let tx = unsafe { CHANNEL_TX.assume_init_ref() }.lock();

    let title: String = env.get_string(jtitle).unwrap().into();

    tx.send(RenderMessage::SetTitle(title));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_bakeBlockModels(
    env: JNIEnv,
    class: JClass) -> jobject {

    let renderer = unsafe { RENDERER.assume_init_mut() };

    let block_hashmap = env.new_object("java/util/HashMap", "()V", &[])
        .unwrap();

    renderer.mc.bake_blocks(renderer);
    println!("Baked block models");

    renderer.mc.block_manager.read().baked_block_variants.iter().for_each(|(identifier, (key, _))| {
        let _integer = env.new_object("java/lang/Integer", "(I)V", &[
            JValue::Int(*key as i32)
        ]).unwrap();

        env.call_method(block_hashmap, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[
            JValue::Object(env.new_string(identifier.to_string()).unwrap().into()),
            JValue::Object(_integer.into())
        ]).unwrap();
    });

    block_hashmap.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_setWorldRenderState(
    env: JNIEnv,
    class: JClass,
    boolean: jboolean) {

    let render_state = unsafe { MC_STATE.assume_init_ref() };
    render_state.write().render_world = boolean != 0;
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_uploadChunkSimple(
    env: JNIEnv,
    class: JClass,
    blocks: jintArray,
    x: jint,
    z: jint) {
    let renderer = unsafe { RENDERER.assume_init_ref() };
    
    let elements = env.get_int_array_elements(blocks, ReleaseMode::NoCopyBack)
        .unwrap();
    
    let elements_ptr = elements.as_ptr();
    let blocks: Box<[BlockState; CHUNK_VOLUME]> = (0..elements.size().unwrap()).map(|index| {
        let element_ptr = unsafe { elements_ptr.offset(index as isize) };
        let block_index = unsafe { *element_ptr } as usize;
    
        BlockState {
            packed_key: Some(block_index as u32)
        }
    }).collect::<Box<[BlockState]>>().try_into().unwrap();

    let mut chunk = Chunk::new((x, z), blocks);
    let baked = BakedChunk::bake(renderer, &chunk);

    chunk.baked = Some(baked);
    renderer.mc.chunks.loaded_chunks.insert((x, z), ArcSwap::new(Arc::new(chunk)));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_genTexture(
    env: JNIEnv,
    class: JClass
) -> jint {
    unsafe { gl::gen_texture() as i32 }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_genBuffer(
    env: JNIEnv,
    class: JClass
) -> jint {
    unsafe { gl::gen_buffer() as i32 }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_bindTexture(
    env: JNIEnv,
    class: JClass,
    tex: jint
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::BindTexture(tex));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_bindBuffer(
    env: JNIEnv,
    class: JClass,
    target: jint,
    buffer: jint
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::BindBuffer(target, buffer));
}


#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_activeTexture(
    env: JNIEnv,
    class: JClass,
    tex: jint
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::ActiveTexture(tex));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_pushMatrix(
    env: JNIEnv,
    class: JClass
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::PushMatrix);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_popMatrix(
    env: JNIEnv,
    class: JClass
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::PopMatrix);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_submitCommands(
    env: JNIEnv,
    class: JClass
) {
    // println!("{:?}", unsafe {GL_COMMANDS.assume_init_ref()});

    unsafe { GL_PIPELINE.assume_init_mut() }.commands.store(
        Arc::new(
            (*unsafe {GL_COMMANDS.assume_init_ref()}).clone()
        )
    );
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_vertexPointer(
    env: JNIEnv,
    class: JClass,
    size: jint,
    vertex_type: jint,
    stride: jint,
    pointer: jlong
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::VertexPointer(size, vertex_type, stride, pointer as u64));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_colorPointer(
    env: JNIEnv,
    class: JClass,
    size: jint,
    vertex_type: jint,
    stride: jint,
    pointer: jlong
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::ColorPointer(size, vertex_type, stride, pointer as u64));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_texCoordPointer(
    env: JNIEnv,
    class: JClass,
    size: jint,
    vertex_type: jint,
    stride: jint,
    pointer: jlong
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::TexCoordPointer(size, vertex_type, stride, pointer as u64));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_drawArray(
    env: JNIEnv,
    class: JClass,
    mode: jint,
    first: jint,
    count: jint
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::DrawArray(mode, first, count));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_texImage2D(
    env: JNIEnv,
    class: JClass,
    target: jint,
    level: jint,
    internal_format: jint,
    width: jint,
    height: jint,
    border: jint,
    format: jint,
    _type: jint,
    pixels_ptr: jlong
) {
    let area = width * height;
    //In bytes
    assert_eq!(_type, 0x1401);
    let size = area as usize * 24;

    let data = if pixels_ptr != 0 {
        Vec::from(
            unsafe {
                std::slice::from_raw_parts(pixels_ptr as *const u8, size)
            }
        )
    } else {
        vec![0; size]
    };

    let wm = unsafe { RENDERER.assume_init_ref() };

    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::TexImage2D((
        RefCell::new(
            Some(data)
        ),
        width as u32,
        height as u32,
        match format {
            0x1908 => wgpu::TextureFormat::Rgba8UnormSrgb,
            0x80E1 => wgpu::TextureFormat::Bgra8UnormSrgb,
            _ => panic!("Unknown format {:x}", format)
        }
    )));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getMaxTextureSize(
    env: JNIEnv,
    class: JClass,
) -> jint {
    let wm = unsafe { RENDERER.assume_init_ref() };
    wm.wgpu_state.adapter.limits().max_texture_dimension_2d as i32
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getWindowWidth(
    env: JNIEnv,
    class: JClass,
) -> jint {
    unsafe { WINDOW.assume_init_ref().inner_size().width as i32 }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_getWindowHeight(
    env: JNIEnv,
    class: JClass,
) -> jint {
    unsafe { WINDOW.assume_init_ref().inner_size().height as i32 }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_enableClientState(
    env: JNIEnv,
    class: JClass,
    int: jint
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::EnableClientState(int as u32));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_WgpuNative_disableClientState(
    env: JNIEnv,
    class: JClass,
    int: jint
) {
    let commands = unsafe { gl::GL_COMMANDS.assume_init_mut() };
    commands.push(GLCommand::DisableClientState(int as u32));
}