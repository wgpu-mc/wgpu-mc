use std::collections::HashMap;
use std::mem::size_of;
use std::thread;
use std::time::Duration;
use std::{sync::Arc, time::Instant};

use futures::executor::block_on;
use jni::{
    objects::{JString, JValue},
    JNIEnv,
};
use winit::event_loop::EventLoopBuilder;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, ModifiersState, WindowEvent},
    event_loop::ControlFlow,
};

use wgpu_mc::render::graph::{GeometryCallback, ShaderGraph};
use wgpu_mc::render::shaderpack::ShaderPackConfig;
use wgpu_mc::wgpu;
use wgpu_mc::{render::atlas::Atlas, WmRenderer};

use crate::gl::{electrum_gui_callback, ElectrumVertex};
use crate::{
    entity::ENTITY_ATLAS, MinecraftResourceManagerAdapter, RenderMessage, WinitWindowWrapper,
    CHANNELS, MC_STATE, RENDERER, WINDOW,
};

pub fn start_rendering(env: JNIEnv, title: JString) {
    let title: String = env.get_string(title).unwrap().into();

    // Hacky fix for starting the game on linux, needs more investigation (thanks, accusitive)
    // https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopBuilder.html#method.build
    let mut event_loop = EventLoopBuilder::new();
    #[cfg(target_os = "linux")]
    {
        use winit::platform::unix::EventLoopBuilderExtUnix;
        event_loop.with_any_thread(true);
    }
    let event_loop = event_loop.build();

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

    println!("Opened window");

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

    let _ = RENDERER.set(wm.clone());

    wm.init();

    env.set_static_field(
        "dev/birb/wgpu/render/Wgpu",
        ("dev/birb/wgpu/render/Wgpu", "INITIALIZED", "Z"),
        JValue::Bool(true.into()),
    )
    .unwrap();

    let mut current_modifiers = ModifiersState::empty();

    println!("Starting event loop");

    let wm_clone = wm.clone();
    let wm_clone_1 = wm.clone();

    let shader_pack: ShaderPackConfig =
        serde_yaml::from_str(include_str!("../graph.yaml")).unwrap();
    let mut shader_graph = ShaderGraph::new(shader_pack);

    let mut types = HashMap::new();

    types.insert("wm_electrum_gl_texture".into(), "texture".into());

    let mut geometry = HashMap::new();

    geometry.insert(
        "wm_geo_electrum_gui".into(),
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ElectrumVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ElectrumVertex::VAO,
        },
    );

    shader_graph.init(&wm, Some(&types), Some(geometry));

    thread::spawn(move || {
        let wm = wm_clone;

        loop {
            wm.upload_camera();

            let mc_state = MC_STATE.load();

            let surface_state = wm.wgpu_state.surface.read();

            let surface = surface_state.0.as_ref().unwrap();
            let texture = surface.get_current_texture().unwrap();

            let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: Default::default(),
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            let _instant = Instant::now();

            let resources = HashMap::new();

            let mut geometry = HashMap::new();
            let key = String::from("wm_geo_electrum_gui");

            let callback = Box::new(&electrum_gui_callback) as GeometryCallback;

            geometry.insert(&key, &callback);

            wm.render(&shader_graph, Some(&resources), Some(&geometry), &view)
                .unwrap();

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
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        wm.resize(wgpu_mc::WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height,
                        });
                    }
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        ..
                    } => {
                        CHANNELS
                            .0
                            .send(RenderMessage::MouseMove(position.x, position.y))
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
                    WindowEvent::ReceivedCharacter(c) => {
                        CHANNELS
                            .0
                            .send(RenderMessage::CharTyped(*c, current_modifiers.bits()))
                            .unwrap();
                    }
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input,
                        is_synthetic: _,
                    } => {
                        // input.scancode
                        match input.virtual_keycode {
                            None => {}
                            Some(keycode) => CHANNELS
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
            // Event::RedrawRequested(_) => {

            // }
            _ => {}
        }
    });
}
