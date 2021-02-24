use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use std::sync::{Mutex, RwLock, mpsc, Arc};
use wgpu_mc::{Renderer, WindowSize, HasWindowSize, ShaderProvider};
use std::mem::MaybeUninit;
use glfw::{Context, Key, Action, Window};
use std::{thread, fs};
use std::sync::mpsc::{channel, RecvError};
use std::ops::Deref;
use futures::executor::block_on;
use glfw::ffi::GLFWwindow;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::path::Path;

enum WindowMessage {
    SetTitle(Arc<String>)
}

static mut RENDERER: MaybeUninit<Mutex<Renderer>> = MaybeUninit::uninit();
static mut CHANNEL_TX: MaybeUninit<mpsc::Sender<WindowMessage>> = MaybeUninit::uninit();

struct GlfwWindowWrapper {
    handle: RawWindowHandle,
    size: WindowSize
}

impl HasWindowSize for GlfwWindowWrapper {
    fn get_window_size(&self) -> WindowSize {
        self.size
    }
}

unsafe impl HasRawWindowHandle for GlfwWindowWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.handle
    }
}

struct SimpleShaderProvider {

}

impl ShaderProvider for SimpleShaderProvider {
    fn get_shader(&self, name: &str) -> Vec<u8> {
        fs::read(
            Path::new(
                "/home/birb/wgpu-mc/wgpu-mc-demo/res/shaders"
            ).join(name)
        ).unwrap() //very basic
    }
}

#[no_mangle]
pub extern "system" fn Java_cloud_birb_wgpu_rust_Wgpu_initializeWindow(env: JNIEnv, class: JClass, string: JString) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (tx, rx) = channel();

    unsafe {
        CHANNEL_TX = MaybeUninit::new(tx);
    }

    let vm = env.get_java_vm().unwrap();
    let title: String = env.get_string(string).unwrap().into();

    thread::spawn(move || {
        let env = vm.attach_current_thread_permanently().unwrap();

        let (mut window, events) = glfw.create_window(640, 480, &title[..], glfw::WindowMode::Windowed).unwrap();

        let renderer = block_on(Renderer::new(
            &GlfwWindowWrapper {
                handle: window.raw_window_handle(),
                size: WindowSize {
                    width: 640,
                    height: 480
                }
            }, Box::new(SimpleShaderProvider {})
        ));

        while !window.should_close() {
            // Swap front and back buffers
            window.swap_buffers();

            // Poll for and process events
            glfw.poll_events();

            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true)
                    },
                    _ => {},
                }
            }

            match rx.try_recv() {
                Ok(message) => match message {
                    WindowMessage::SetTitle(title) => {
                        window.set_title(&title.deref().clone()[..]);
                    }
                }
                Err(_) => {}
            }
        }
    });
}

#[no_mangle]
pub extern "system" fn Java_cloud_birb_wgpu_rust_Wgpu_updateWindowTitle(env: JNIEnv, class: JClass, string: JString) {
    let input: String = env.get_string(string).unwrap().into();

    unsafe {
        (*(CHANNEL_TX.as_mut_ptr())).send(
            WindowMessage::SetTitle(Arc::new(input))
        );
    }
}