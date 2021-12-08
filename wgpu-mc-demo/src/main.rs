use std::{iter, fs};
use std::path::PathBuf;
use std::ops::{DerefMut, Deref};
use std::time::Instant;
use wgpu_mc::mc::datapack::{TagOrResource, NamespacedResource, BlockModel};
use wgpu_mc::mc::block::{BlockDirection, BlockState, Block};
use wgpu_mc::mc::chunk::{ChunkSection, Chunk, CHUNK_AREA, CHUNK_HEIGHT, CHUNK_SECTION_HEIGHT, CHUNK_SECTIONS_PER, CHUNK_VOLUME};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState, DeviceEvent};
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
use wgpu_mc::mc::block::model::BlockstateVariantMesh;

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

    let block_id = NamespacedResource::try_from("anvil.json").unwrap();
    let key = block_manager.get_packed_blockstate_key(&block_id, "facing=east");
    // let anvil_model: &BlockModel = block_manager.models.get(&NamespacedResource::try_from("block/cobblestone").unwrap()).unwrap();
    let mesh: &BlockstateVariantMesh = block_manager.baked_block_variants.get(
        &NamespacedResource::try_from("cobblestone.json").unwrap()
    ).unwrap();

    let model = block_manager.models.get(
        &NamespacedResource::try_from("block/cobblestone")
            .unwrap()
    ).unwrap();

    println!("Mesh {:?}\n\nModel {:?}", mesh, model);


    let blocks: Box<[BlockState; CHUNK_VOLUME]> = (0..CHUNK_VOLUME).map(|block| {
        BlockState {
            packed_key: if block == 0 { key } else { None },
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

    let mut frame_start = Instant::now();
    let mut frame_time = 1.0;

    let mut forward = 0.0;

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

                                //removed atm
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                forward = 1.0;
                            },
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                forward = 0.0;
                            },
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            } => {
                                forward = -1.0;
                            },
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            } => {
                                forward = 0.0;
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
                        },
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                &state.update();

                frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                let mut camera = **state.mc.camera.load();

                let direction = camera.get_direction();

                println!("{}", frame_time);

                camera.position += direction * 200.0 * frame_time * forward;

                state.mc.camera.store(Arc::new(camera));

                &state.render(&[
                    &WorldPipeline {}
                ]);

                frame_start = Instant::now();
            },
            Event::DeviceEvent {
                ref event,
                ..
            } => {
                match event {
                    // DeviceEvent::Added => {}
                    // DeviceEvent::Removed => {}
                    DeviceEvent::MouseMotion { delta } => {
                        let mut camera = **state.mc.camera.load();
                        camera.yaw += (delta.0 / 100.0) as f32;
                        camera.pitch -= (delta.1 / 100.0) as f32;
                        state.mc.camera.store(Arc::new(camera));
                    },
                    _ => {},
                }
            },
            _ => {}
        }
    });
}