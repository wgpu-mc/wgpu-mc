use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use std::sync::{Mutex, RwLock, mpsc, Arc};
use wgpu_mc::Renderer;
use std::mem::MaybeUninit;
use glfw::{Context, Key, Action, Window};
use std::thread;
use std::sync::mpsc::{channel, RecvError};
use std::ops::Deref;

enum WindowMessage {
    SetTitle(Arc<String>)
}

static mut RENDERER: MaybeUninit<Mutex<Renderer>> = MaybeUninit::uninit();
static mut CHANNEL_TX: MaybeUninit<mpsc::Sender<WindowMessage>> = MaybeUninit::uninit();

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