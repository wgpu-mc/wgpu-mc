mod mc_interface;

use jni::JNIEnv;
use jni::objects::{JClass, JString, JValue, JObject};
use jni::sys::{jstring, jint, jobjectArray};
use std::sync::{Mutex, RwLock, mpsc, Arc};
use wgpu_mc::{Renderer, WindowSize, HasWindowSize, ShaderProvider};
use std::mem::MaybeUninit;
use std::{thread, fs};
use std::sync::mpsc::{channel, RecvError};
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
use wgpu_mc::mc::chunk::CHUNK_HEIGHT;
use crate::mc_interface::xyz_to_index;
use wgpu_mc::mc::datapack::NamespacedId;
use std::convert::TryFrom;

#[derive(Debug)]
enum RenderMessage {
    SetTitle(String),
    RegisterSprite(String)
}

static mut RENDERER: MaybeUninit<Renderer> = MaybeUninit::uninit();
static mut EVENT_LOOP: MaybeUninit<EventLoop<()>> = MaybeUninit::uninit();
static mut WINDOW: MaybeUninit<Window> = MaybeUninit::uninit();

static mut CHANNEL_TX: MaybeUninit<mpsc::Sender<RenderMessage>> = MaybeUninit::uninit();
static mut CHANNEL_RX: MaybeUninit<mpsc::Receiver<RenderMessage>> = MaybeUninit::uninit();

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

struct SimpleShaderProvider {}

impl ShaderProvider for SimpleShaderProvider {
    fn get_shader(&self, name: &str) -> String {
        String::from_utf8(fs::read(Path::new("/Users/birb/wgpu-mc")
            .join("res").join("shaders").join(name))
            .expect("Couldn't locate the shaders")).unwrap()
        //very basic
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
    jnamespace: JString) {

    let tx = unsafe { CHANNEL_TX.assume_init_mut() };
    let namespace: String = env.get_string(jnamespace).unwrap().into();

    tx.send(RenderMessage::RegisterSprite(namespace));
}

#[no_mangle]
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_registerEntry(
    env: JNIEnv,
    class: JClass,
    entry_type: jint,
    name: JString) {

    let rname: String = env.get_string(name).unwrap().into();

    let renderer = unsafe { RENDERER.assume_init_mut() };

    match entry_type {
        0 => renderer.registry.blocks.insert(rname),
        1 => renderer.registry.items.insert(rname),
        _ => unimplemented!()
    };

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

    let mut state = block_on(Renderer::new(
        &wrapper, Box::new(SimpleShaderProvider {})
    ));

    let (tx, rx) = mpsc::channel::<RenderMessage>();

    unsafe {
        CHANNEL_TX = MaybeUninit::new(tx);
        CHANNEL_RX = MaybeUninit::new(rx);
        RENDERER = MaybeUninit::new(state);
        EVENT_LOOP = MaybeUninit::new(event_loop);
        WINDOW = MaybeUninit::new(window);
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

    let rx = unsafe { CHANNEL_RX.assume_init_mut() };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        let (mut state, window) = unsafe {
            (
                RENDERER.assume_init_mut(),
                WINDOW.assume_init_ref()
            )
        };

        let mut msg_r = rx.try_recv();
        while msg_r.is_ok() {
            let msg = msg_r.unwrap();

            println!("{:?}", msg);
            match msg {
                RenderMessage::SetTitle(title) => window.set_title(&title),
                RenderMessage::RegisterSprite(sprite) => {
                    state.mc.texture_manager.textures.insert(
                        NamespacedId::try_from(sprite.as_str()).unwrap(),
                        None
                    );
                }
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
                &state.update();
                // &state.render(&[]);

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
pub extern "system" fn Java_dev_birb_wgpu_rust_Wgpu_updateWindowTitle(
    env: JNIEnv,
    class: JClass,
    jtitle: JString) {

    let tx = unsafe { CHANNEL_TX.assume_init_mut() };
    let title: String = env.get_string(jtitle).unwrap().into();

    tx.send(RenderMessage::SetTitle(title));
}