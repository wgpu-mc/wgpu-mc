use std::thread;
use std::{sync::Arc, time::Instant};

use arc_swap::ArcSwap;
use futures::executor::block_on;
use jni::{
    objects::{JString, JValue},
    JNIEnv,
};
use rayon::ThreadPoolBuilder;
use winit::event_loop::EventLoopBuilder;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, ModifiersState, WindowEvent},
    event_loop::ControlFlow,
};

use wgpu_mc::wgpu;
use wgpu_mc::{
    render::atlas::Atlas,
    render::pipeline::{debug_lines::DebugLinesPipeline, terrain::TerrainPipeline, WmPipeline},
    WmRenderer,
};

use crate::{
    entity::ENTITY_ATLAS, MinecraftRenderState, MinecraftResourceManagerAdapter, RenderMessage,
    WinitWindowWrapper, CHANNELS, GL_PIPELINE, MC_STATE, RENDERER, THREAD_POOL, WINDOW,
};

pub fn start_rendering(env: JNIEnv, title: JString) {
    let title: String = env.get_string(title).unwrap().into();

    THREAD_POOL
        .set(ThreadPoolBuilder::new().num_threads(0).build().unwrap())
        .unwrap();

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

    MC_STATE
        .set(ArcSwap::new(Arc::new(MinecraftRenderState {
            render_world: false,
        })))
        .unwrap();

    let wrapper = &WinitWindowWrapper { window: &window };

    let wgpu_state = block_on(WmRenderer::init_wgpu(wrapper));

    let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
        jvm: env.get_java_vm().unwrap(),
    });

    let wm = WmRenderer::new(wgpu_state, resource_provider);

    let _ = RENDERER.set(wm.clone());

    wm.init(&[
        &DebugLinesPipeline,
        &TerrainPipeline,
        GL_PIPELINE.get().unwrap(),
    ]);

    wm.mc.chunks.assemble_world_meshes(&wm);

    env.set_static_field(
        "dev/birb/wgpu/render/Wgpu",
        ("dev/birb/wgpu/render/Wgpu", "INITIALIZED", "Z"),
        JValue::Bool(true.into()),
    )
    .unwrap();

    let mut current_modifiers = ModifiersState::empty();

    println!("Starting event loop");

    let wm_clone = wm.clone();

    thread::spawn(move || {
        let wm = wm_clone;

        loop {
            wm.upload_camera();

            let mc_state = MC_STATE.get().unwrap().load();

            let mut pipelines = Vec::new();
            pipelines.push(&TerrainPipeline as &dyn WmPipeline);

            if mc_state.render_world {
                // wm.update_animated_textures((SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() / 50) as u32);
                pipelines.push(&DebugLinesPipeline as &dyn WmPipeline);
            } else {
                pipelines.push(GL_PIPELINE.get().unwrap());
            }

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

            wm.render(&pipelines, &view).unwrap();
            // // println!("Frametime: {}ms", Instant::now().duration_since(instant).as_millis());

            texture.present();
        }
    });

    ENTITY_ATLAS
        .set(Arc::new(Atlas::new(
            &wm.wgpu_state,
            &wm.render_pipeline_manager.load(),
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
                            .get()
                            .unwrap()
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
                            .get()
                            .unwrap()
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
                            .get()
                            .unwrap()
                            .0
                            .send(RenderMessage::MouseState(*state, *button))
                            .unwrap();
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        CHANNELS
                            .get()
                            .unwrap()
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
                                .get()
                                .unwrap()
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
