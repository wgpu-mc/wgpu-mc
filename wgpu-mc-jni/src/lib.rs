#![feature(once_cell)]

///Foreword here,
/// the code is quite messy and will be cleaned up at some point.
/// I'm sure there are even some bad practices atm but the general goal is to get things working
/// relatively well, and then clean them up, while still keeping the prototyped code updateable and
/// managable. Feel free to make a pull request.

mod mc_interface;

use jni::{JNIEnv, JavaVM};
use jni::objects::{JClass, JString, JValue, JObject, ReleaseMode};
use jni::sys::{jstring, jint, jobjectArray, jbyteArray, jboolean, jobject, jintArray};
use std::sync::{mpsc, Arc};
use wgpu_mc::{WmRenderer, WindowSize, HasWindowSize};
use std::mem::MaybeUninit;
use std::{thread, fs};
use std::sync::mpsc::{channel, RecvError, Sender};
use std::ops::Deref;
use futures::executor::block_on;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::path::Path;
use winit::window::Window;
use glfw::WindowEvent::Size;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use std::time::Instant;
use winit::event_loop::{ControlFlow, EventLoop};
use wgpu_mc::mc::chunk::{CHUNK_HEIGHT, Chunk, CHUNK_VOLUME};
use crate::mc_interface::xyz_to_index;
use wgpu_mc::mc::datapack::{TagOrResource, NamespacedResource};
use std::convert::{TryFrom, TryInto};
use std::lazy::OnceCell;
use wgpu_mc::mc::BlockEntry;
use wgpu_mc::mc::resource::{ResourceProvider};
use parking_lot::{RwLock, Mutex};
use jni::errors::Error;
use wgpu_mc::render::pipeline::default::WorldPipeline;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::mc::block::{BlockState, BlockDirection};
use wgpu_mc::render::chunk::BakedChunk;
use arc_swap::ArcSwap;

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
            Err(e) => panic!("{:?}", e)
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
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_uploadChunk(
    env: JNIEnv,
    class: JClass,
    world_chunk: JObject) {

    mc_interface::chunk_from_java_world_chunk(&env, &world_chunk);

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_getBackend(
    env: JNIEnv,
    class: JClass) -> jstring {

    let renderer = unsafe { RENDERER.assume_init_ref() };
    let backend = renderer.get_backend_description();

    env.new_string(backend)
         .unwrap()
        .into_inner()

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_registerSprite(
    env: JNIEnv,
    class: JClass,
    sprite_type: JString,
    jnamespace: JString,
    buffer: jbyteArray) {

    let renderer = unsafe { RENDERER.assume_init_ref() };

    let tx = unsafe { CHANNEL_TX.assume_init_ref() }.lock();
    let namespace_str: String = env.get_string(jnamespace).unwrap().into();
    let sprite_type: String = env.get_string(sprite_type).unwrap().into();
    let identifier = NamespacedResource::try_from(namespace_str.as_str())
        .unwrap();

    let buffer_arr = env.get_byte_array_elements(
        buffer,
        ReleaseMode::NoCopyBack)
        .unwrap();

    let buffer_arr_size = buffer_arr.size().unwrap() as usize;
    let mut buffer: Vec<u8> = vec![0; buffer_arr_size];
    //Create a slice looking into the buffer so that we can immediately copy the data into a Vec.
    let slice: &[u8] = unsafe {
        std::mem::transmute(
            std::slice::from_raw_parts(
                buffer_arr.as_ptr(),
                buffer_arr_size)
        )
    };

    buffer.copy_from_slice(slice);

    renderer.mc.texture_manager.textures.insert(identifier, buffer);
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_registerEntry(
    env: JNIEnv,
    class: JClass,
    entry_type: jint,
    name: JString) {

    let rname: String = env.get_string(name).unwrap().into();

    let renderer = unsafe { RENDERER.assume_init_mut() };

    let mut block_manager = renderer.mc.block_manager.write();

    // match entry_type {
    //     0 => block_manager.register_block(TagOrResource::try_from(rname.as_str()).unwrap()),
    //     // 1 => renderer.registry.items.insert(rname),
    //     _ => unimplemented!()
    // };

}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_initialize(
    env: JNIEnv,
    class: JClass,
    string: JString
) {
    use winit::event_loop::EventLoop;

    let title: String = env.get_string(string).unwrap().into();

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title(&format!("{} + wgpu-mc", title))
        .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
            width: 600,
            height: 500
        }))
        .build(&event_loop)
        .unwrap();

    let wrapper = &WinitWindowWrapper {
        window: &window
    };

    let mut state = block_on(WmRenderer::new(
        &wrapper, Arc::new(MinecraftResourceManagerAdapter {
            jvm: env.get_java_vm().unwrap()
        }))
    );

    let (tx, rx) = mpsc::channel::<RenderMessage>();

    unsafe {
        CHANNEL_TX = MaybeUninit::new(Mutex::new(tx));
        CHANNEL_RX = MaybeUninit::new(Mutex::new(rx));
        RENDERER = MaybeUninit::new(state);
        EVENT_LOOP = MaybeUninit::new(event_loop);
        WINDOW = MaybeUninit::new(window);
        MC_STATE = MaybeUninit::new(RwLock::new(MinecraftRenderState {
            render_world: false
        }));
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_doEventLoop(
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
                if !state.input(event) {
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
            }
            Event::RedrawRequested(_) => {
                state.update();
                let mut pipelines: Vec<&dyn WmPipeline> = Vec::with_capacity(1);
                if mc_state.render_world {
                    pipelines.push(&WorldPipeline {});
                }
                state.render(&pipelines[..]);

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
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_digestInputStream(
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
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_updateWindowTitle(
    env: JNIEnv,
    class: JClass,
    jtitle: JString) {

    let tx = unsafe { CHANNEL_TX.assume_init_ref() }.lock();

    let title: String = env.get_string(jtitle).unwrap().into();

    tx.send(RenderMessage::SetTitle(title));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_bakeBlockModels(
    env: JNIEnv,
    class: JClass) -> jobject {

    let renderer = unsafe { RENDERER.assume_init_mut() };

    renderer.mc.bake_blocks();
    println!("Generated blocks");

    let block_hashmap = env.new_object("java/util/HashMap", "()V", &[])
        .unwrap();

    renderer.mc.block_manager.read().block_models.iter().for_each(|(id, entry)| {
        println!("block id {}", id);

        let identifier_string = env.new_string(id.to_string())
            .unwrap();
        let index = entry.index.unwrap();
        let _integer = env.new_object("java/lang/Integer", "(I)V", &[
            JValue::Int(index as i32)
        ]).unwrap();

        env.call_method(block_hashmap, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[
            JValue::Object(identifier_string.into()),
            JValue::Object(_integer.into())
        ]).unwrap();
    });

    renderer.mc.generate_block_texture_atlas(&renderer.wgpu_state.device, &renderer.wgpu_state.queue, &renderer.pipelines.load().layouts.texture_bind_group_layout);

    block_hashmap.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_setWorldRenderState(
    env: JNIEnv,
    class: JClass,
    boolean: jboolean) {

    let render_state = unsafe { MC_STATE.assume_init_ref() };
    render_state.write().render_world = boolean != 0;
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_uploadChunkSimple(
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
            block: Some(block_index),
            direction: BlockDirection::North,
            damage: 0,
            transparency: false
        }
    }).collect::<Box<[BlockState]>>().try_into().unwrap();

    let mut chunk = Chunk::new((x, z), blocks);
    let baked = BakedChunk::bake(renderer, &chunk);

    chunk.baked = Some(baked);
    renderer.mc.chunks.loaded_chunks.insert((x, z), ArcSwap::new(Arc::new(chunk)));
}