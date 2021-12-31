use std::{iter, fs};
use std::path::PathBuf;
use std::ops::{DerefMut, Deref};
use std::time::Instant;
use wgpu_mc::mc::datapack::{TextureVariableOrResource, NamespacedResource, BlockModel};
use wgpu_mc::mc::block::{BlockDirection, BlockState, Block};
use wgpu_mc::mc::chunk::{ChunkSection, Chunk, CHUNK_AREA, CHUNK_HEIGHT, CHUNK_SECTION_HEIGHT, CHUNK_SECTIONS_PER, CHUNK_VOLUME, CHUNK_WIDTH};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState, DeviceEvent};
use wgpu_mc::{WmRenderer, HasWindowSize, WindowSize};
use futures::executor::block_on;
use winit::window::Window;
use cgmath::InnerSpace;
use std::sync::Arc;
use wgpu_mc::mc::resource::{ResourceProvider};
use std::convert::{TryFrom, TryInto};
use wgpu_mc::render::chunk::BakedChunk;
use wgpu_mc::render::pipeline::builtin::WorldPipeline;
use arc_swap::ArcSwap;
use futures::StreamExt;
use std::collections::HashMap;
use wgpu_mc::mc::block::model::BlockstateVariantMesh;
use wgpu_mc::util::WmArena;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use fastanvil::{RegionBuffer};
use std::io::Cursor;
use fastanvil::pre18::JavaChunk;
use rayon::iter::{IntoParallelRefIterator, IntoParallelIterator};
use fastnbt::de::from_bytes;

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

struct Test {
    thing: String
}

impl Drop for Test {
    fn drop(&mut self) {
        println!("dropping test {:?}", self.thing);
    }
}

fn load_anvil_chunks() -> Vec<(usize, usize, JavaChunk)> {
    let root = crate_root::root().unwrap().join("wgpu-mc-demo").join("res");
    let demo_root = root.join("demo_world");
    let region_dir = std::fs::read_dir(
        demo_root.join("region")
    ).unwrap();
    // let mut model_map = HashMap::new();

    let begin = Instant::now();

    let regions: Vec<Vec<u8>> = region_dir.map(|region| {
        let region = region.unwrap();
        fs::read(region.path()).unwrap()
    }).collect();

    use rayon::iter::ParallelIterator;
    regions
        .into_par_iter()
        .flat_map(|region| {
            let cursor = Cursor::new(region);
            let mut region = RegionBuffer::new(cursor);
            let mut chunks = Vec::new();
            region.for_each_chunk(|x, z, chunk_data| {
                let chunk: JavaChunk = from_bytes(&chunk_data[..]).unwrap();
                chunks.push((x, z, chunk));
            });
            chunks
        }).collect()
}

fn main() {
    let anvil_chunks = load_anvil_chunks();

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

    println!("Testing arena");

    {
        let mut arena = WmArena::new(1024);
        for _ in 0..10000 {
            arena.alloc(
                String::from("Testing arena")
            );
        }
    }

    println!("Arena ok");

    println!("Generating blocks");
    wm.mc.bake_blocks(&wm);

    let window = wrapper.window;

    println!("Starting rendering");
    begin_rendering(event_loop, window, wm, anvil_chunks);
}

fn begin_rendering(mut event_loop: EventLoop<()>, mut window: Window, mut state: WmRenderer, chunks: Vec<(usize, usize, JavaChunk)>) {
    use futures::executor::block_on;

    let block_manager = state.mc.block_manager.read();

    let mc_state = state.mc.clone();
    let wgpu_state = state.wgpu_state.clone();

    println!("Chunks: {}", chunks.len());
    // println!("Blocks {:?}", block_manager.baked_block_variants);

    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::ParallelIterator;
    chunks.into_par_iter().take(1).for_each(|(chunk_x, chunk_z, java_chunk): (usize, usize, JavaChunk)| {
        let mut chunk_blocks = Box::new([BlockState {
            packed_key: None
        }; CHUNK_VOLUME]);
        (0..16).zip((0..256).zip(0..16)).for_each(|(x,(y,z))| {
            use fastanvil::Chunk;
            let block_maybe = java_chunk.block(x as usize, y as isize, z as usize);
            match block_maybe {
                None => {}
                Some(block) => {
                    // let variant = block.encoded_description().replace("|", "#");
                    let splits = block.encoded_description().split_once("|")
                        .unwrap();
                    let mut variant = splits.0.to_string();
                    variant.push_str(".json#");
                    variant.push_str(splits.1);

                    chunk_blocks[
                        (x + (z * CHUNK_WIDTH)) + (y * CHUNK_AREA)
                    ] = BlockState {
                        packed_key: Some(*block_manager.baked_block_variants.get_with_key(
                            &NamespacedResource::try_from(&variant[..]).unwrap()
                        ).expect(&variant[..]).0)
                    }
                }
            }
        });
        let mut chunk = Chunk::new((chunk_x as i32, chunk_z as i32), chunk_blocks);
        let bm = mc_state.block_manager.read();
        let wgpu_state_arc = wgpu_state.clone();
        chunk.bake(&*bm, &wgpu_state_arc.device);
        println!("Baked chunk @ {},{}", chunk_x, chunk_z);

        // mc_state.chunks.loaded_chunks.insert((chunk_x as i32, chunk_z as i32), ArcSwap::new(Arc::new(chunk)));
        mc_state.chunks.loaded_chunks.insert((0, 0), ArcSwap::new(Arc::new(chunk)));
    });

    drop(block_manager);

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
            Event::RedrawRequested(_) => {
                &state.update();

                frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                let mut camera = **state.mc.camera.load();

                let direction = camera.get_direction();
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