use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jstring;
use std::sync::{Mutex, RwLock};
use wgpu_mc::Renderer;
use std::mem::MaybeUninit;
use glfw::{Context, Key, Action};

static mut RENDERER: MaybeUninit<Mutex<Renderer>> = MaybeUninit::uninit();

#[no_mangle]
pub extern "system" fn Java_cloud_birb_wgpu_rust_Wgpu_initializeWindow(env: JNIEnv, class: JClass) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(640, 480, "Minecraft + wgpu-mc", glfw::WindowMode::Windowed).unwrap();

    window.make_current();
    window.set_key_polling(true);

    while !window.should_close() {
        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                _ => {},
            }
        }
    }
}