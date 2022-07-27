mod chunk;
mod entity;

#[macro_use]
extern crate wgpu_mc;

use std::{fs, thread};

use std::path::PathBuf;

use std::time::{Instant, SystemTime, UNIX_EPOCH};
use wgpu_mc::mc::resource::ResourcePath;

use futures::executor::block_on;
use wgpu_mc::{HasWindowSize, wgpu, WindowSize, WmRenderer};
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
use wgpu_mc::mc::entity::{EntityInstanceTransforms, PartTransform};

use wgpu_mc::render::pipeline::debug_lines::DebugLinesPipeline;
use wgpu_mc::render::pipeline::entity::EntityPipeline;
use wgpu_mc::render::pipeline::sky::SkyPipeline;
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
    fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>> {
        let real_path = self.asset_root.join(id.0.replace(":", "/"));

        fs::read(&real_path).ok()
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

fn main() {
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
        &EntityPipeline { entities: &[] },
        &TerrainPipeline,
        &SkyPipeline,
        &TransparentPipeline,
        &DebugLinesPipeline,
    ]);

    let blockstates_path = _mc_root.join("blockstates");

    let blocks = {
        //Read all of the blockstates in the Minecraft datapack folder thingy
        let blockstate_dir = std::fs::read_dir(blockstates_path).unwrap();
        // let mut model_map = HashMap::new();
        let mut bm = wm.mc.block_manager.write();

        blockstate_dir.map(|m| {
            let model = m.unwrap();
            (
                format!("minecraft:{}", model.file_name().to_str().unwrap().replace(".json", "")),
                format!("minecraft:blockstates/{}", model.file_name().to_str().unwrap()).into()
            )
        })
    }.collect::<Vec<_>>();

    let now = Instant::now();

    wm.mc.bake_blocks(&wm, blocks.iter().map(|(a,b)| (a, b)));

    let end = Instant::now();

    println!("Baked {} blocks in {}ms", wm.mc.block_manager.read().blocks.len(), end.duration_since(now).as_millis());

    let window = wrapper.window;

    begin_rendering(event_loop, window, wm);
}

fn begin_rendering(
    event_loop: EventLoop<()>,
    window: Window,
    wm: WmRenderer
) {
    let (entity, mut instances) = describe_entity(&wm);

    let chunk = make_chunks(&wm);

    {
        wm.mc.chunks.loaded_chunks.write().insert((0, 0), ArcSwap::new(Arc::new(chunk)));
    }

    // wm.mc.chunks.bake_meshes(&wm, provider);
    wm.mc.chunks.assemble_world_meshes(&wm);

    let mut frame_start = Instant::now();
    let mut frame_time = 1.0;

    let mut forward = 0.0;

    let mut spin: f32 = 0.0;

    let mut frame: u32 = 0;

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

                wm.update_animated_textures((SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() / 50) as u32);

                frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                spin += 0.5;
                frame += 1;

                let mut camera = **wm.mc.camera.load();

                let direction = camera.get_direction();
                camera.position += direction * 200.0 * frame_time * forward;

                wm.mc.camera.store(Arc::new(camera));

                let surface = wm.wgpu_state.surface.as_ref().unwrap();
                let texture = surface.get_current_texture().unwrap();
                let view = texture.texture.create_view(
                    &wgpu::TextureViewDescriptor {
                        label: None,
                        format: Some(wgpu::TextureFormat::Bgra8Unorm),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: Default::default(),
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None
                    }
                );

                let _ = wm.render(
                    &[
                        // &SkyPipeline,
                        &TerrainPipeline,
                        // &GrassPipeline,
                        // &TransparentPipeline,
                        &EntityPipeline {
                            entities: &[
                                &instances
                            ]
                        },
                        &DebugLinesPipeline
                    ],
                    &view
                );

                texture.present();

                instances.instances = (0..1).map(
                    |id| {
                        EntityInstanceTransforms {
                            // position: ((id / 10) as f32 * 5.0, 0.0, (id % 10) as f32 * 5.0),
                            position: (0.0, 0.0, 0.0),
                            looking_yaw: 0.0,
                            uv_offset: (0.0, 0.0),
                            part_transforms: vec![
                                PartTransform {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                    pivot_x: 0.0,
                                    pivot_y: 0.0,
                                    pivot_z: 0.0,
                                    yaw: 0.0,
                                    pitch: 0.0,
                                    roll: 0.0,
                                    scale_x: 1.0,
                                    scale_y: 1.0,
                                    scale_z: 1.0
                                },
                                PartTransform {
                                    x: 0.0,
                                    y: ((spin / 20.0).sin() * 0.5) as f32,
                                    // y: 0.0,
                                    z: 0.0,
                                    pivot_x: 0.5,
                                    pivot_y: 0.5,
                                    pivot_z: 0.5,
                                    yaw: spin + 30.0 + (id as f32 + 5.0),
                                    pitch: spin + 30.0 + (id as f32 + 5.0),
                                    roll: spin + 30.0 + (id as f32 + 5.0),
                                    scale_x: 1.0,
                                    scale_y: 1.0,
                                    scale_z: 1.0
                                },
                                PartTransform {
                                    x: 0.0,
                                    y: ((spin / 20.0).sin() * 0.5) as f32,
                                    // y: 0.0,
                                    z: 0.0,
                                    pivot_x: 0.6,
                                    pivot_y: 0.6,
                                    pivot_z: 0.6,
                                    yaw: spin + (id as f32 + 5.0),
                                    pitch: spin + (id as f32 + 5.0),
                                    roll: spin + (id as f32 + 5.0),
                                    scale_x: 1.0,
                                    scale_y: 1.0,
                                    scale_z: 1.0
                                },
                                PartTransform {
                                    x: 0.0,
                                    y: ((spin / 20.0).sin() * 0.5) as f32,
                                    // y: 0.0,
                                    z: 0.0,
                                    pivot_x: 0.6,
                                    pivot_y: 0.6,
                                    pivot_z: 0.6,
                                    yaw: spin + 10.0 + (id as f32 + 5.0),
                                    pitch: spin + 50.0 + (id as f32 + 5.0),
                                    roll: spin + 150.0 + (id as f32 + 5.0),
                                    scale_x: 1.0,
                                    scale_y: 1.0,
                                    scale_z: 1.0
                                }
                            ]
                        }
                    }
                ).collect();

                instances.upload(&wm);

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
