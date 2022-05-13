#[macro_use]
extern crate wgpu_mc;

mod chunk;
mod entity;

use std::fs;

use std::path::PathBuf;

use std::time::Instant;
use wgpu_mc::mc::datapack::NamespacedResource;

use futures::executor::block_on;
use wgpu_mc::{HasWindowSize, WindowSize, WmRenderer};
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use std::sync::Arc;
use wgpu_mc::mc::resource::ResourceProvider;

use futures::StreamExt;

use fastanvil::RegionBuffer;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::io::Cursor;
use arc_swap::ArcSwap;

use fastanvil::pre18::JavaChunk;
use fastnbt::de::from_bytes;
use rayon::iter::IntoParallelIterator;
use wgpu_mc::mc::block::Block;

use wgpu_mc::render::pipeline::debug_lines::DebugLinesPipeline;
use wgpu_mc::render::pipeline::entity::EntityPipeline;
use wgpu_mc::render::pipeline::terrain::TerrainPipeline;
use wgpu_mc::render::pipeline::transparent::TransparentPipeline;
use wgpu_mc::render::pipeline::WmPipeline;

use crate::chunk::make_chunks;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_mc::wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
};
// use crate::chunk::make_chunks;
use crate::entity::describe_entity;

struct FsResourceProvider {
    pub asset_root: PathBuf,
}

//ResourceProvider is what wm uses to fetch resources. This is a basic implementation that's just backed by the filesystem
impl ResourceProvider for FsResourceProvider {
    fn get_resource(&self, id: &NamespacedResource) -> Option<Vec<u8>> {
        let real_path = self.asset_root.join(&id.0).join(&id.1);
        if !real_path.exists() { return None; }

        Some(fs::read(&real_path)
            .unwrap_or_else(|_| panic!("{}", real_path.to_str().unwrap().to_string())))
    }
}

struct WinitWindowWrapper {
    window: Window,
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

fn load_anvil_chunks() -> Vec<(usize, usize, JavaChunk)> {
    let root = crate_root::root().unwrap().join("wgpu-mc-demo").join("res");
    let demo_root = root.join("demo_world");
    let region_dir = std::fs::read_dir(demo_root.join("region")).unwrap();
    // let mut model_map = HashMap::new();

    let _begin = Instant::now();

    let regions: Vec<Vec<u8>> = region_dir
        .map(|region| {
            let region = region.unwrap();
            fs::read(region.path()).unwrap()
        })
        .collect();

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
        })
        .collect()
}

fn main() {
    let anvil_chunks = load_anvil_chunks();

    let event_loop = EventLoop::new();
    let title = "wgpu-mc test";
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let wrapper = WinitWindowWrapper { window };

    let rsp = Arc::new(FsResourceProvider {
        asset_root: crate_root::root()
            .unwrap()
            .join("wgpu-mc-demo")
            .join("res")
            .join("assets"),
    });

    let _mc_root = crate_root::root()
        .unwrap()
        .join("wgpu-mc-demo")
        .join("res")
        .join("assets")
        .join("minecraft");

    let wgpu_state = block_on(WmRenderer::init_wgpu(&wrapper));

    let wm = WmRenderer::new(wgpu_state, rsp);

    wm.init(&[
        &EntityPipeline { frames: &[] },
        &TerrainPipeline,
        &TransparentPipeline,
        &DebugLinesPipeline,
    ]);

    let blockstates_path = _mc_root.join("blockstates");

    {
        let blockstate_dir = std::fs::read_dir(blockstates_path).unwrap();
        // let mut model_map = HashMap::new();
        let mut bm = wm.mc.block_manager.write();

        blockstate_dir.for_each(|m| {
            let model = m.unwrap();

            let resource_name = NamespacedResource(
                String::from("minecraft"),
                format!("blockstates/{}", model.file_name().to_str().unwrap()),
            );

            match Block::from_json(
                model.file_name().to_str().unwrap(),
                std::str::from_utf8(&fs::read(model.path()).unwrap()).unwrap(),
            ) {
                None => {}
                Some(block) => {
                    bm.blocks.insert(resource_name, block);
                }
            };
        });
    }

    // println!("Generating blocks");
    wm.mc.bake_blocks(&wm);

    let window = wrapper.window;

    println!("Starting rendering");
    begin_rendering(event_loop, window, wm, anvil_chunks);
}

fn begin_rendering(
    event_loop: EventLoop<()>,
    window: Window,
    wm: WmRenderer,
    _chunks: Vec<(usize, usize, JavaChunk)>,
) {
    // let entity_rendering = describe_entity(&wm);

    let chunks = make_chunks(&wm);

    {
        let mut loaded_chunks = wm.mc.chunks.loaded_chunks.write();

        chunks.into_iter().for_each(|chunk| {
            loaded_chunks.insert(chunk.pos, ArcSwap::new(Arc::new(chunk)));
        });
    }

    wm.mc.chunks.assemble_world_meshes(&wm);

    let mut frame_start = Instant::now();
    let mut frame_time = 1.0;

    let mut forward = 0.0;

    let mut spin = 0.0;

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
                        }
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        } => {
                            forward = 1.0;
                        }
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        } => {
                            forward = 0.0;
                        }
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::S),
                            ..
                        } => {
                            forward = -1.0;
                        }
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
                        let _ = wm.resize(WindowSize {
                            width: physical_size.width,
                            height: physical_size.height,
                        });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let _ = wm.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height,
                        });
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                wm.upload_camera();

                frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                spin += 0.5;

                let mut camera = **wm.mc.camera.load();

                let direction = camera.get_direction();
                camera.position += direction * 200.0 * frame_time * forward;

                wm.mc.camera.store(Arc::new(camera));

                let _ = wm.render(&[
                    &TerrainPipeline,
                    // &GrassPipeline,
                    &TransparentPipeline,
                    &DebugLinesPipeline,
                ]);

                frame_start = Instant::now();
            }
            // Event::DeviceEvent { event: DeviceEvent::Added {..}, ..} => {}
            // Event::DeviceEvent { event: DeviceEvent::Removed {..}, ..} => {}
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let mut camera = **wm.mc.camera.load();
                camera.yaw += (delta.0 / 100.0) as f32;
                camera.pitch -= (delta.1 / 100.0) as f32;
                wm.mc.camera.store(Arc::new(camera));
            }
            _ => {}
        }
    });
}
