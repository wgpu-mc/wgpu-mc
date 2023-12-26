use std::collections::HashMap;
use std::io::Cursor;
use std::mem::size_of;
use std::{slice, thread};
use std::{sync::Arc, time::Instant};

use byteorder::LittleEndian;
use cgmath::{perspective, Deg, Matrix4, SquareMatrix};
use futures::executor::block_on;
use jni::objects::{AutoElements, JClass, JFloatArray, ReleaseMode};
use jni::sys::{jfloat, jint};
use jni::{
    objects::{JString, JValue},
    JNIEnv,
};
use jni_fn::jni_fn;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::{Mutex, RwLock};
use winit::event::{DeviceEvent, Ime, KeyEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, WindowEvent},
};

use wgpu_mc::mc::block::{BlockMeshVertex, BlockstateKey};
use wgpu_mc::mc::chunk::RenderLayer;
use wgpu_mc::render::graph::{CustomResource, GeometryCallback, ResourceInternal, ShaderGraph};
use wgpu_mc::render::pipeline::Vertex;
use wgpu_mc::render::shaderpack::{Mat4, Mat4ValueOrMult, ShaderPackConfig};
use wgpu_mc::util::BindableBuffer;
use wgpu_mc::wgpu;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_mc::wgpu::{BufferUsages, TextureFormat};
use wgpu_mc::{render::atlas::Atlas, WmRenderer};

use crate::gl::{ElectrumGeometry, ElectrumVertex};
use crate::{
    entity::ENTITY_ATLAS, MinecraftResourceManagerAdapter, RenderMessage, WinitWindowWrapper,
    CHANNELS, MC_STATE, RENDERER, WINDOW,
};

pub static MATRICES: Lazy<Mutex<Matrices>> = Lazy::new(|| {
    Mutex::new(Matrices {
        projection: [[0.0; 4]; 4],
        view: [[0.0; 4]; 4],
    })
});

static SHOULD_STOP: OnceCell<()> = OnceCell::new();

pub struct Matrices {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}

pub struct TerrainLayer;

impl RenderLayer for TerrainLayer {
    fn filter(&self) -> fn(BlockstateKey) -> bool {
        |_| true
    }

    fn mapper(&self) -> fn(&BlockMeshVertex, f32, f32, f32) -> Vertex {
        |vert, x, y, z| Vertex {
            position: [
                vert.position[0] + x,
                vert.position[1] + y,
                vert.position[2] + z,
            ],
            uv: vert.tex_coords,
            normal: [vert.normal[0], vert.normal[1], vert.normal[2]],
            color: u32::MAX,
            uv_offset: vert.animation_uv_offset,
        }
    }

    fn name(&self) -> &str {
        "all"
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setChunkOffset(_env: JNIEnv, _class: JClass, x: jint, z: jint) {
    *RENDERER.get().unwrap().mc.chunks.chunk_offset.lock() = [x, z];
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn setMatrix(mut env: JNIEnv, _class: JClass, _id: jint, float_array: JFloatArray) {
    let elements: AutoElements<jfloat> =
        unsafe { env.get_array_elements(&float_array, ReleaseMode::NoCopyBack) }.unwrap();

    let slice = unsafe { slice::from_raw_parts(elements.as_ptr(), elements.len()) };

    let mut cursor = Cursor::new(bytemuck::cast_slice::<f32, u8>(slice));
    let mut converted = Vec::with_capacity(slice.len());

    for _ in 0..slice.len() {
        use byteorder::ReadBytesExt;
        converted.push(cursor.read_f32::<LittleEndian>().unwrap());
    }

    let slice_4x4: [[f32; 4]; 4] = *bytemuck::from_bytes(bytemuck::cast_slice(&converted));
    MATRICES.lock().projection = slice_4x4;
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn scheduleStop(_env: JNIEnv, _class: JClass) {
    let _ = SHOULD_STOP.set(());
}

pub fn start_rendering(mut env: JNIEnv, title: JString) {
    let title: String = env.get_string(&title).unwrap().into();

    // Hacky fix for starting the game on linux, needs more investigation (thanks, accusitive)
    // https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopBuilder.html#method.build
    let mut event_loop = EventLoopBuilder::new();

    #[cfg(target_os = "linux")]
    {
        // double hacky fix B)
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            event_loop.with_any_thread(true);
        } else {
            use winit::platform::x11::EventLoopBuilderExtX11;
            event_loop.with_any_thread(true);
        }
    }

    let event_loop = event_loop.build().unwrap();

    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
                width: 1280,
                height: 720,
            }))
            .build(&event_loop)
            .unwrap(),
    );

    log::info!("Opened window");

    WINDOW.set(window.clone()).unwrap();

    let wrapper = &WinitWindowWrapper { window: &window };

    let wgpu_state = block_on(WmRenderer::init_wgpu(
        wrapper,
        super::SETTINGS.read().as_ref().unwrap().vsync.value,
    ));

    let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
        jvm: env.get_java_vm().unwrap(),
    });

    let wm = WmRenderer::new(wgpu_state, resource_provider);

    wm.pipelines
        .load()
        .chunk_layers
        .store(Arc::new(vec![Box::new(TerrainLayer)]));

    let _ = RENDERER.set(wm.clone());

    wm.init();

    env.set_static_field(
        "dev/birb/wgpu/render/Wgpu",
        ("dev/birb/wgpu/render/Wgpu", "initialized", "Z"),
        JValue::Bool(true.into()),
    )
    .unwrap();

    let mut current_modifiers = ModifiersState::empty();

    log::trace!("Starting event loop");

    let wm_clone = wm.clone();

    let shader_pack: ShaderPackConfig =
        serde_yaml::from_str(include_str!("../graph.yaml")).unwrap();

    let mut render_geometry = HashMap::new();

    render_geometry.insert(
        "wm_geo_electrum_gui".into(),
        Box::new(ElectrumGeometry {
            blank: wm.create_texture_handle(
                "electrum_blank_texture".into(),
                TextureFormat::Bgra8Unorm,
                &wm.wgpu_state.surface.read().1,
            ),
            pool: Arc::new(
                wm.wgpu_state
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: &vec![0u8; 10_000_000],
                        usage: BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
                    }),
            ),
            last_bytes: RwLock::new(None),
        }) as Box<dyn GeometryCallback>,
    );

    let mut resources = HashMap::new();

    let matrix = Matrix4::identity();
    let mat: Mat4 = matrix.into();
    let bindable_buffer = BindableBuffer::new(
        &wm,
        bytemuck::cast_slice(&mat),
        BufferUsages::UNIFORM,
        "matrix",
    );

    resources.insert(
        "wm_mat4_projection".into(),
        CustomResource {
            update: None,
            data: Arc::new(ResourceInternal::Mat4(
                Mat4ValueOrMult::Value { value: mat },
                Arc::new(RwLock::new(matrix)),
                Arc::new(bindable_buffer),
            )),
        },
    );

    let matrix = Matrix4::identity();
    let mat: Mat4 = matrix.into();
    let bindable_buffer = BindableBuffer::new(
        &wm,
        bytemuck::cast_slice(&mat),
        BufferUsages::UNIFORM,
        "matrix",
    );

    resources.insert(
        "wm_mat4_view".into(),
        CustomResource {
            update: None,
            data: Arc::new(ResourceInternal::Mat4(
                Mat4ValueOrMult::Value { value: mat },
                Arc::new(RwLock::new(matrix)),
                Arc::new(bindable_buffer),
            )),
        },
    );

    let mut shader_graph = ShaderGraph::new(shader_pack, resources, render_geometry);

    let mut types = HashMap::new();

    types.insert("wm_electrum_mat4".into(), "matrix".into());
    types.insert("wm_electrum_gl_texture".into(), "texture".into());

    let mut geometry_layouts = HashMap::new();

    geometry_layouts.insert(
        "wm_geo_electrum_gui".into(),
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ElectrumVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ElectrumVertex::VAO,
        },
    );

    shader_graph.init(&wm, Some(&types), Some(geometry_layouts));

    thread::spawn(move || {
        let wm = wm_clone;

        loop {
            let _mc_state = MC_STATE.load();

            let surface_state = wm.wgpu_state.surface.read();

            {
                let matrices = MATRICES.lock();
                let res_mat_proj = shader_graph
                    .resources
                    .get_mut("wm_mat4_projection")
                    .unwrap();

                if let ResourceInternal::Mat4(_val, lock, _) = &*res_mat_proj.data {
                    let matrix4: Matrix4<f32> = matrices.projection.into();
                    *lock.write() = perspective(
                        Deg(100.0),
                        (surface_state.1.width as f32) / (surface_state.1.height as f32),
                        0.01,
                        1000.0,
                    ) * matrix4;
                }
            }

            let surface = surface_state.0.as_ref().unwrap();
            let texture = surface.get_current_texture().unwrap();

            let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: Default::default(),
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            let _instant = Instant::now();

            wm.render(&shader_graph, &view, &surface_state.1).unwrap();

            texture.present();
        }
    });

    ENTITY_ATLAS
        .set(Arc::new(Atlas::new(
            &wm.wgpu_state,
            &wm.pipelines.load(),
            false,
        )))
        .unwrap();

    event_loop
        .run(move |event, target| {
            if SHOULD_STOP.get().is_some() {
                target.exit();
            }

            match event {
                Event::AboutToWait => window.request_redraw(),
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            wm.resize(wgpu_mc::WindowSize {
                                width: physical_size.width,
                                height: physical_size.height,
                            });
                            CHANNELS
                                .0
                                .send(RenderMessage::Resized(
                                    physical_size.width,
                                    physical_size.height,
                                ))
                                .unwrap();
                        }
                        WindowEvent::MouseInput {
                            device_id: _,
                            state,
                            button,
                            ..
                        } => {
                            CHANNELS
                                .0
                                .send(RenderMessage::MouseState(*state, *button))
                                .unwrap();
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            CHANNELS
                                .0
                                .send(RenderMessage::CursorMove(position.x, position.y))
                                .unwrap();
                        }
                        WindowEvent::Ime(Ime::Commit(s)) => {
                            // could have multiple chars, see winit docs
                            // TODO check if this still works as intended
                            // TODO might want to move the for loop to the java side? to avoid multiple jni calls
                            for c in s.chars() {
                                CHANNELS
                                    .0
                                    .send(RenderMessage::CharTyped(
                                        c,
                                        modifiers_to_glfw(current_modifiers),
                                    ))
                                    .unwrap();
                            }
                        }
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(key),
                                    state,
                                    ..
                                },
                            ..
                        } => {
                            if let Some(scancode) = key.to_scancode() {
                                CHANNELS
                                    .0
                                    .send(RenderMessage::KeyState(
                                        keycode_to_glfw(*key),
                                        scancode,
                                        match state {
                                            ElementState::Pressed => 1,  // GLFW_PRESS
                                            ElementState::Released => 0, // GLFW_RELEASE
                                        },
                                        modifiers_to_glfw(current_modifiers),
                                    ))
                                    .unwrap();
                            }
                        }
                        WindowEvent::ModifiersChanged(new_modifiers) => {
                            current_modifiers = new_modifiers.state();
                        }
                        WindowEvent::Focused(focused) => {
                            CHANNELS.0.send(RenderMessage::Focused(*focused)).unwrap();
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent {
                    device_id: _,
                    event: DeviceEvent::MouseMotion { delta },
                } => {
                    CHANNELS
                        .0
                        .send(RenderMessage::MouseMove(delta.0, delta.1))
                        .unwrap();
                }
                _ => {}
            }
        })
        .unwrap();
}

fn keycode_to_glfw(code: KeyCode) -> u32 {
    match code {
        KeyCode::Space => 32,
        KeyCode::Quote => 39,
        KeyCode::Comma => 44,
        KeyCode::Minus => 45,
        KeyCode::Period => 46,
        KeyCode::Slash => 47,
        KeyCode::Digit0 => 48,
        KeyCode::Digit1 => 49,
        KeyCode::Digit2 => 50,
        KeyCode::Digit3 => 51,
        KeyCode::Digit4 => 52,
        KeyCode::Digit5 => 53,
        KeyCode::Digit6 => 54,
        KeyCode::Digit7 => 55,
        KeyCode::Digit8 => 56,
        KeyCode::Digit9 => 57,
        KeyCode::Semicolon => 59,
        KeyCode::Equal => 61,
        KeyCode::KeyA => 65,
        KeyCode::KeyB => 66,
        KeyCode::KeyC => 67,
        KeyCode::KeyD => 68,
        KeyCode::KeyE => 69,
        KeyCode::KeyF => 70,
        KeyCode::KeyG => 71,
        KeyCode::KeyH => 72,
        KeyCode::KeyI => 73,
        KeyCode::KeyJ => 74,
        KeyCode::KeyK => 75,
        KeyCode::KeyL => 76,
        KeyCode::KeyM => 77,
        KeyCode::KeyN => 78,
        KeyCode::KeyO => 79,
        KeyCode::KeyP => 80,
        KeyCode::KeyQ => 81,
        KeyCode::KeyR => 82,
        KeyCode::KeyS => 83,
        KeyCode::KeyT => 84,
        KeyCode::KeyU => 85,
        KeyCode::KeyV => 86,
        KeyCode::KeyW => 87,
        KeyCode::KeyX => 88,
        KeyCode::KeyY => 89,
        KeyCode::KeyZ => 90,
        KeyCode::BracketLeft => 91,
        KeyCode::Backslash => 92,
        KeyCode::BracketRight => 93,
        KeyCode::Backquote => 96,
        // what the fuck are WORLD_1 (161) and WORLD_2 (162)
        KeyCode::Escape => 256,
        KeyCode::Enter => 257,
        KeyCode::Tab => 258,
        KeyCode::Backspace => 259,
        KeyCode::Insert => 260,
        KeyCode::Delete => 261,
        KeyCode::ArrowRight => 262,
        KeyCode::ArrowLeft => 263,
        KeyCode::ArrowDown => 264,
        KeyCode::ArrowUp => 265,
        KeyCode::PageUp => 266,
        KeyCode::PageDown => 267,
        KeyCode::Home => 268,
        KeyCode::End => 269,
        KeyCode::CapsLock => 280,
        KeyCode::ScrollLock => 281,
        KeyCode::NumLock => 282,
        KeyCode::PrintScreen => 283,
        KeyCode::Pause => 284,
        KeyCode::F1 => 290,
        KeyCode::F2 => 291,
        KeyCode::F3 => 292,
        KeyCode::F4 => 293,
        KeyCode::F5 => 294,
        KeyCode::F6 => 295,
        KeyCode::F7 => 296,
        KeyCode::F8 => 297,
        KeyCode::F9 => 298,
        KeyCode::F10 => 299,
        KeyCode::F11 => 300,
        KeyCode::F12 => 301,
        KeyCode::F13 => 302,
        KeyCode::F14 => 303,
        KeyCode::F15 => 304,
        KeyCode::F16 => 305,
        KeyCode::F17 => 306,
        KeyCode::F18 => 307,
        KeyCode::F19 => 308,
        KeyCode::F20 => 309,
        KeyCode::F21 => 310,
        KeyCode::F22 => 311,
        KeyCode::F23 => 312,
        KeyCode::F24 => 313,
        KeyCode::F25 => 314,
        KeyCode::Numpad0 => 320,
        KeyCode::Numpad1 => 321,
        KeyCode::Numpad2 => 322,
        KeyCode::Numpad3 => 323,
        KeyCode::Numpad4 => 324,
        KeyCode::Numpad5 => 325,
        KeyCode::Numpad6 => 326,
        KeyCode::Numpad7 => 327,
        KeyCode::Numpad8 => 328,
        KeyCode::Numpad9 => 329,
        KeyCode::NumpadDecimal => 330,
        KeyCode::NumpadDivide => 331,
        KeyCode::NumpadMultiply => 332,
        KeyCode::NumpadSubtract => 333,
        KeyCode::NumpadAdd => 334,
        KeyCode::NumpadEnter => 335,
        KeyCode::NumpadEqual => 336,
        KeyCode::ShiftLeft => 340,
        KeyCode::ControlLeft => 341,
        KeyCode::AltLeft => 342,
        KeyCode::SuperLeft => 343,
        KeyCode::ShiftRight => 344,
        KeyCode::ControlRight => 345,
        KeyCode::AltRight => 346,
        KeyCode::SuperRight => 347,
        KeyCode::ContextMenu => 348,
        _ => 0,
    }
}

fn modifiers_to_glfw(state: ModifiersState) -> u32 {
    if state.is_empty() {
        return 0;
    }

    let mut mods = 0;

    if state.shift_key() {
        mods |= 1;
    }
    if state.control_key() {
        mods |= 2;
    }
    if state.alt_key() {
        mods |= 4;
    }
    if state.super_key() {
        mods |= 8;
    }

    mods
}
