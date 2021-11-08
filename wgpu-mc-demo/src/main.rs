use std::{iter, fs};
use std::path::PathBuf;
use std::ops::{DerefMut, Deref};
use std::time::Instant;
use wgpu_mc::mc::datapack::Identifier;
use wgpu_mc::mc::block::{BlockDirection, BlockState};
use wgpu_mc::mc::chunk::{ChunkSection, Chunk, CHUNK_AREA, CHUNK_HEIGHT, CHUNK_SECTION_HEIGHT, CHUNK_SECTIONS_PER, CHUNK_VOLUME};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};
use wgpu_mc::{WmRenderer, ShaderProvider, HasWindowSize, WindowSize};
use futures::executor::block_on;
use winit::window::Window;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use cgmath::InnerSpace;
use std::sync::Arc;
use wgpu_mc::mc::resource::ResourceProvider;
use std::convert::{TryFrom, TryInto};
use wgpu_mc::render::chunk::BakedChunk;
use wgpu_mc::render::pipeline::default::WorldPipeline;

struct SimpleResourceProvider {
    pub asset_root: PathBuf
}

struct SimpleShaderProvider {
    pub shader_root: PathBuf
}

impl ResourceProvider for SimpleResourceProvider {

    fn get_resource(&self, id: &Identifier) -> Vec<u8> {

        let (namespace, path) = match id {
            Identifier::Resource(inner) => {
                inner
            },
            _ => unreachable!()
        };

        let real_path = self.asset_root.join(namespace).join(path);
        fs::read(real_path).unwrap()
    }

}

impl ShaderProvider for SimpleShaderProvider {
    fn get_shader(&self, name: &str) -> String {
        let path = self.shader_root.join(name);
        String::from_utf8(fs::read(path).expect(&format!("Shader {} does not exist", name))).unwrap()
    }
}

struct WinitWindowWrapper {
    window: Window
}

impl HasWindowSize for WinitWindowWrapper {
    fn get_window_size(&self) -> WindowSize {
        WindowSize {
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
        }
    }
}

unsafe impl HasRawWindowHandle for WinitWindowWrapper {

    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }

}

fn main() {
    let event_loop = EventLoop::new();
    let title = "wgpu-mc test";
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let wrapper = WinitWindowWrapper {
        window
    };

    let sp = SimpleShaderProvider {
        shader_root: crate_root::root().unwrap().join("res").join("shaders"),
    };

    let rsp = SimpleResourceProvider {
        asset_root: crate_root::root().unwrap().join("res").join("assets"),
    };

    let mc_root = crate_root::root()
        .unwrap()
        .join("res")
        .join("assets")
        .join("minecraft");

    let mut wm = block_on(
        WmRenderer::new(
            &wrapper,
            Arc::new(rsp),
            Arc::new(sp)
        )
    );

    println!("Loading block models");
    wm.mc.generate_block_models();
    println!("Generating texture atlas");
    wm.mc.generate_block_texture_atlas(&wm.wgpu_state.device, &wm.wgpu_state.queue, &wm.pipelines.read().layouts.texture_bind_group_layout);
    println!("Baking blocks");
    wm.mc.bake_blocks(&wm.wgpu_state.device);

    let window = wrapper.window;

    println!("Starting rendering");
    begin_rendering(event_loop, window, wm);
}

fn begin_rendering(mut event_loop: EventLoop<()>, mut window: Window, mut state: WmRenderer) {
    use futures::executor::block_on;

    let block_manager = state.mc.block_manager.read();
    let anvil = Identifier::try_from("minecraft:block/anvil").unwrap();

    let blocks = (0..CHUNK_SECTIONS_PER).map(|_| {
        BlockState {
            block: block_manager.blocks.get(&anvil).unwrap().index,
            direction: BlockDirection::North,
            damage: 0,
            transparency: false
        }
    }).collect::<Box<[BlockState]>>().try_into().unwrap();

    drop(block_manager);

    let chunk = Chunk::new((0, 0), blocks);

    let instant = Instant::now();
    let baked_chunk = BakedChunk::bake(&state, &chunk);
    println!("Time to generate chunk mesh: {}", Instant::now().duration_since(instant).as_millis());

    let chunks = [chunk];

    let mut frame_begin = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            } => {
                                //Update a block and re-generate the chunk mesh for testing

                                //removed atm for testing
                            },

                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Down),
                                ..
                            } => {
                                state.mc.camera.write().pitch += 0.1;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Up),
                                ..
                            } => {
                                state.mc.camera.write().pitch -= 0.1;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Left),
                                ..
                            } => {
                                state.mc.camera.write().yaw -= 0.1;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Right),
                                ..
                            } => {
                                state.mc.camera.write().yaw += 0.1;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                ..
                            } => {
                                state.mc.camera.write().position.y -= 0.1;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::E),
                                ..
                            } => {
                                state.mc.camera.write().position.y += 0.1;
                            },

                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                let mut camera = state.mc.camera.write();
                                let direction: cgmath::Vector3<f32> = (camera.yaw.cos(), camera.pitch.sin(), camera.yaw.sin()).into();
                                camera.position += direction.normalize();
                            },

                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        },
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
                &state.render(&[
                    &WorldPipeline {}
                ]);

                let delta = Instant::now().duration_since(frame_begin).as_millis()+1; //+1 so we don't divide by zero
                frame_begin = Instant::now();

                // println!("Frametime {}, FPS {}", delta, 1000/delta);
            }
            _ => {}
        }
    });
}