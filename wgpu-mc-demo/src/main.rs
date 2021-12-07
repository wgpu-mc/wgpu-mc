use std::{iter, fs};
use std::path::PathBuf;
use std::ops::{DerefMut, Deref};
use std::time::Instant;
use wgpu_mc::mc::datapack::{TagOrResource, NamespacedResource, BlockModel};
use wgpu_mc::mc::block::{BlockDirection, BlockState, Block};
use wgpu_mc::mc::chunk::{ChunkSection, Chunk, CHUNK_AREA, CHUNK_HEIGHT, CHUNK_SECTION_HEIGHT, CHUNK_SECTIONS_PER, CHUNK_VOLUME};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};
use wgpu_mc::{WmRenderer, HasWindowSize, WindowSize};
use futures::executor::block_on;
use winit::window::Window;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use cgmath::InnerSpace;
use std::sync::Arc;
use wgpu_mc::mc::resource::{ResourceProvider};
use std::convert::{TryFrom, TryInto};
use wgpu_mc::render::chunk::BakedChunk;
use wgpu_mc::render::pipeline::default::WorldPipeline;
use arc_swap::ArcSwap;
use futures::StreamExt;
use std::collections::HashMap;

struct SimpleResourceProvider {
    pub asset_root: PathBuf
}

impl ResourceProvider for SimpleResourceProvider {

    fn get_resource(&self, id: &NamespacedResource) -> Vec<u8> {
        let real_path = self.asset_root.join(&id.0).join(&id.1);
        fs::read(&real_path).expect(real_path.to_str().unwrap())
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

    let rsp = SimpleResourceProvider {
        asset_root: crate_root::root().unwrap().join("wgpu-mc-demo").join("res").join("assets"),
    };

    let mc_root = crate_root::root()
        .unwrap()
        .join("wgpu-mc-demo")
        .join("res")
        .join("assets")
        .join("minecraft");

    let mut wm = block_on(
        WmRenderer::new(
            &wrapper,
            Arc::new(rsp),
        )
    );

    let blockstates_path = mc_root.join("blockstates");

    {
        let blockstate_dir = std::fs::read_dir(blockstates_path).unwrap();
        // let mut model_map = HashMap::new();
        let mut bm = wm.mc.block_manager.write();

        blockstate_dir.for_each(|m| {
            let model = m.unwrap();

            let resource_name = NamespacedResource (
                String::from("minecraft"),
                String::from(model.file_name().to_str().unwrap())
            );

            match Block::from_json(model.file_name().to_str().unwrap(), std::str::from_utf8(&fs::read(model.path()).unwrap()).unwrap()) {
                None => {}
                Some(block) => { bm.blocks.insert(resource_name, block); }
            };
        });
    }

    let bm = wm.mc.block_manager.read();

    // bm.block_models.iter().for_each(|block| {
    //     block.1.model.textures.iter().for_each(|(_, texture_identifier)| {
    //         if matches!(texture_identifier, TagOrResource::Resource(_)) {
    //             wm.mc.texture_manager.insert_texture(texture_identifier.clone(), wm.mc.resource_provider.get_resource(&texture_identifier.append(".png")))
    //         }
    //     });
    // });

    drop(bm);

    println!("Generating blocks");
    wm.mc.bake_blocks(&wm);

    // wm.mc.bake_blockstate_meshes(&wm.wgpu_state.device);

    let window = wrapper.window;

    println!("Starting rendering");
    begin_rendering(event_loop, window, wm);
}

fn begin_rendering(mut event_loop: EventLoop<()>, mut window: Window, mut state: WmRenderer) {
    use futures::executor::block_on;

    let block_manager = state.mc.block_manager.read();

    let block_id = NamespacedResource::try_from("minecraft:block/cobblestone").unwrap();

    let blocks = (0..CHUNK_VOLUME).map(|_| {
        BlockState {
            packed_key: block_manager.get_packed_blockstate_key(&block_id, ""),
        }
    }).collect::<Box<[BlockState]>>().try_into().unwrap();

    drop(block_manager);

    // println!("")

    let mut chunk = Chunk::new((0, 0), blocks);

    let instant = Instant::now();
    let baked_chunk = BakedChunk::bake(&state, &chunk);
    println!("Time to generate chunk mesh: {}", Instant::now().duration_since(instant).as_millis());

    chunk.baked = Some(baked_chunk);

    state.mc.chunks.loaded_chunks.insert((0, 0), ArcSwap::new(Arc::new(chunk)));

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
                                let mut camera = **state.mc.camera.load();
                                camera.pitch += 0.01;
                                state.mc.camera.store(Arc::new(camera));
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Up),
                                ..
                            } => {
                                let mut camera = **state.mc.camera.load();
                                camera.pitch -= 0.01;
                                state.mc.camera.store(Arc::new(camera));
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Left),
                                ..
                            } => {
                                let mut camera = **state.mc.camera.load();
                                camera.yaw -= 0.01;
                                state.mc.camera.store(Arc::new(camera));
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Right),
                                ..
                            } => {
                                let mut camera = **state.mc.camera.load();
                                camera.yaw += 0.01;
                                state.mc.camera.store(Arc::new(camera));
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                ..
                            } => {
                                let mut camera = **state.mc.camera.load();
                                camera.position.y -= 0.01;
                                state.mc.camera.store(Arc::new(camera));
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::E),
                                ..
                            } => {
                                let mut camera = **state.mc.camera.load();
                                camera.position.y += 0.01;
                                state.mc.camera.store(Arc::new(camera));
                            },

                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                let mut camera = **state.mc.camera.load();

                                let direction: cgmath::Vector3<f32> = (camera.yaw.cos(), camera.pitch.sin(), camera.yaw.sin()).into();
                                camera.position += direction.normalize();

                                state.mc.camera.store(Arc::new(camera));
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

                let start = Instant::now(); //+1 so we don't divide by zero

                &state.render(&[
                    &WorldPipeline {}
                ]);

                let delta = Instant::now().duration_since(start).as_micros();

                // println!("Frametime {}μs, FPS {}", delta, 1000000/delta);
            }
            _ => {}
        }
    });
}