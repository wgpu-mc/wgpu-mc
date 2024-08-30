use std::sync::Arc;

use futures::executor::block_on;
use jni::{objects::JValue, JavaVM};
use once_cell::sync::OnceCell;
use parking_lot::lock_api::{Mutex, RwLock};
use wgpu_mc::{render::graph::Geometry, wgpu::{self, util::{BufferInitDescriptor, DeviceExt}, BufferAddress, BufferBindingType, PresentMode, TextureFormat}, Display, Frustum, WmRenderer};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::{DeviceEvent, ElementState, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, ModifiersState, PhysicalKey}, platform::scancode::PhysicalKeyExtScancode};

use crate::{gl::{ElectrumGeometry, ElectrumVertex}, renderer::MATRICES, MinecraftResourceManagerAdapter, RenderMessage, CHANNELS, CUSTOM_GEOMETRY, RENDERER, RENDER_GRAPH, SCENE};
use wgpu_mc::render::{shaderpack::ShaderPackConfig,graph::{RenderGraph,ResourceBacking}};
use std::collections::HashMap;


pub static SHOULD_STOP: OnceCell<()> = OnceCell::new();

pub struct Application{
    title:String,
    current_modifiers:ModifiersState,
    jvm:JavaVM,
}
impl Application{
    pub fn new(jvm: JavaVM, title: String)->Self{
        
        
        let current_modifiers = ModifiersState::empty();
        // {
        //     let tex_id = LIGHTMAP_GLID.lock().unwrap();
        //     let textures_read = GL_ALLOC.read();
        //     let lightmap = textures_read.get(&*tex_id).unwrap();
        //     let bindable = lightmap.bindable_texture.as_ref().unwrap();
        //     let asaa = ArcSwap::new(bindable.clone());
        // }
        
        profiling::register_thread!("Winit Thread");
        Self{
            title,
            current_modifiers,
            jvm,
        }
    }
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        log::trace!("Starting event loop");

        let mut env = self.jvm.attach_current_thread().unwrap();

        // initialisation, should only occure once on desktop
        let window_attributes = 
        winit::window::Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
                width: 1280,
                height: 720,
            }));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let size = window.inner_size();


        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        const VSYNC: bool = true;
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: if VSYNC {
                PresentMode::AutoVsync
            } else{
                PresentMode::AutoNoVsync
            },
    
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
    

        let required_limits = wgpu::Limits {
            max_push_constant_size: 128,
            max_bind_groups: 8,
            max_storage_buffers_per_shader_stage: 1000,
            ..Default::default()
        };

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default()
                    | wgpu::Features::DEPTH_CLIP_CONTROL
                    | wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::BUFFER_BINDING_ARRAY
                    | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
                    | wgpu::Features::MULTI_DRAW_INDIRECT,
                required_limits,
                memory_hints:wgpu::MemoryHints::Performance,
            },
            None, // Trace path
        ))
        .unwrap();

        surface.configure(&device, &surface_config);

        let display = Display{
            window,
            size:RwLock::new(size),
            surface,
            device,
            queue,
            config: RwLock::new(surface_config),
            instance,
            adapter,
        };

        
        let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
            jvm: env.get_java_vm().unwrap(),
        });
    

        let wm = WmRenderer::new(display, resource_provider);
    
    
        wm.init();

        let shader_pack: ShaderPackConfig =
            serde_yaml::from_str(include_str!("../graph.yaml")).unwrap();
    
        let mut render_resources = HashMap::new();
    
        let mat4_projection = create_matrix_buffer(&wm);
        let mat4_view = create_matrix_buffer(&wm);
        let mat4_model = create_matrix_buffer(&wm);
    
        render_resources.insert(
            "@mat4_view".into(),
            ResourceBacking::Buffer(mat4_view.clone(), BufferBindingType::Uniform),
        );
    
        render_resources.insert(
            "@mat4_perspective".into(),
            ResourceBacking::Buffer(mat4_projection.clone(), BufferBindingType::Uniform),
        );
    
        render_resources.insert(
            "@mat4_model".into(),
            ResourceBacking::Buffer(mat4_model.clone(), BufferBindingType::Uniform),
        );
    
        let mut custom_bind_groups = HashMap::new();
        custom_bind_groups.insert(
            "@texture_electrum_gui".into(),
            wm.bind_group_layouts.get("texture").unwrap(),
        );
        custom_bind_groups.insert(
            "@mat4_electrum_gui".into(),
            wm.bind_group_layouts.get("matrix").unwrap(),
        );
    
        let mut custom_geometry = HashMap::new();
        custom_geometry.insert(
            "@geo_electrum_gui".into(),
            vec![wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ElectrumVertex>() as BufferAddress,
                step_mode: Default::default(),
                attributes: &ElectrumVertex::VAO,
            }],
        );
    
        let render_graph = RenderGraph::new(
            &wm,
            shader_pack,
            render_resources,
            Some(custom_bind_groups),
            Some(custom_geometry),
        );
        RENDER_GRAPH.set(render_graph);
        let mut geometry = HashMap::new();
        geometry.insert(
            "@geo_electrum_gui".to_string(),
            Box::new(ElectrumGeometry {
                pool: Arc::new(
                    wm.display
                        .device
                        .create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: &vec![0; 1_000_000],
                            usage: wgpu::BufferUsages::COPY_DST
                                | wgpu::BufferUsages::VERTEX
                                | wgpu::BufferUsages::INDEX,
                        }),
                ),
                last_bytes: None,
            }) as Box<dyn Geometry>,
        );
        CUSTOM_GEOMETRY.set(Mutex::new(geometry));

        let _ = RENDERER.set(wm);
        env.set_static_field(
            "dev/birb/wgpu/render/Wgpu",
            ("dev/birb/wgpu/render/Wgpu", "initialized", "Z"),
            JValue::Bool(true.into()),
        )
        .unwrap();

    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion{ delta } => {
                CHANNELS
                    .0
                    .send(RenderMessage::MouseMove(delta.0, delta.1))
                    .unwrap();

            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if SHOULD_STOP.get().is_some() {
            event_loop.exit();
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let wm = RENDERER.get().unwrap();
        if window_id == wm.display.window.id() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    // Update the wgpu_state size for the render loop.
                    *wm.display.size.write() = physical_size;

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
                        .send(RenderMessage::MouseState(state, button))
                        .unwrap();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    CHANNELS
                        .0
                        .send(RenderMessage::CursorMove(position.x, position.y))
                        .unwrap();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            text,
                            state,
                            ..
                        },
                    ..
                } => {
                    if let Some(scancode) = key.to_scancode() {
                        CHANNELS
                            .0
                            .send(RenderMessage::KeyState(
                                keycode_to_glfw(key),
                                scancode,
                                match state {
                                    ElementState::Pressed => 1,  // GLFW_PRESS
                                    ElementState::Released => 0, // GLFW_RELEASE
                                },
                                modifiers_to_glfw(self.current_modifiers),
                            ))
                            .unwrap();

                        if let Some(text) = text {
                            for c in text.chars() {
                                CHANNELS
                                    .0
                                    .send(RenderMessage::CharTyped(
                                        c,
                                        modifiers_to_glfw(self.current_modifiers),
                                    ))
                                    .unwrap();
                            }
                        }
                    }
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    self.current_modifiers = new_modifiers.state();
                }
                WindowEvent::Focused(focused) => {
                    CHANNELS.0.send(RenderMessage::Focused(focused)).unwrap();
                }
                _ => {}
            }
        }
    }
}


fn create_matrix_buffer(wm: &WmRenderer) -> Arc<wgpu::Buffer> {
    Arc::new(
        wm.display
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &[0; 64],
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            }),
    )
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