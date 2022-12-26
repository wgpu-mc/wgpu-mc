extern crate wgpu_mc;

use arc_swap::access::{Access, DynAccess};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use arc_swap::ArcSwap;
use bytemuck::Pod;
use futures::executor::block_on;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use wgpu_mc::mc::block::{BlockMeshVertex, BlockstateKey};
use wgpu_mc::mc::chunk::RenderLayer;
use wgpu_mc::mc::entity::{EntityInstanceTransforms, PartTransform};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::mc::resource::ResourceProvider;
use wgpu_mc::render::graph::{CustomResource, ResourceInternal, ShaderGraph};
use wgpu_mc::render::pipeline::Vertex;
use wgpu_mc::render::shaderpack::Mat4ValueOrMult;
use wgpu_mc::util::UniformStorage;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_mc::wgpu::{
    BindGroupDescriptor, BindGroupEntry, CommandEncoderDescriptor, ComputePassDescriptor, Maintain,
    MaintainBase, MapMode,
};
use wgpu_mc::{wgpu, HasWindowSize, WindowSize, WmRenderer};

use crate::chunk::make_chunks;
use crate::entity::describe_entity;

mod chunk;
mod entity;

struct FsResourceProvider {
    pub asset_root: PathBuf,
}

//ResourceProvider is what wm uses to fetch resources. This is a basic implementation that's just backed by the filesystem
impl ResourceProvider for FsResourceProvider {
    fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>> {
        let real_path = self.asset_root.join(id.0.replace(':', "/"));

        fs::read(real_path).ok()
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

unsafe impl HasRawDisplayHandle for WinitWindowWrapper {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

fn main() {
    env_logger::init();

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

    let wgpu_state = block_on(WmRenderer::init_wgpu(&wrapper, false));

    let wm = WmRenderer::new(wgpu_state, rsp);

    wm.init();

    let blockstates_path = _mc_root.join("blockstates");

    let blocks = {
        //Read all of the blockstates in the Minecraft datapack folder thingy
        let blockstate_dir = fs::read_dir(blockstates_path).unwrap();
        // let mut model_map = HashMap::new();
        let _bm = wm.mc.block_manager.write();

        blockstate_dir.map(|m| {
            let model = m.unwrap();
            (
                format!(
                    "minecraft:{}",
                    model.file_name().to_str().unwrap().replace(".json", "")
                ),
                format!(
                    "minecraft:blockstates/{}",
                    model.file_name().to_str().unwrap()
                )
                .into(),
            )
        })
    }
    .collect::<Vec<_>>();

    let now = Instant::now();

    wm.mc.bake_blocks(&wm, blocks.iter().map(|(a, b)| (a, b)));

    let end = Instant::now();

    println!(
        "Baked {} blocks in {}ms",
        wm.mc.block_manager.read().blocks.len(),
        end.duration_since(now).as_millis()
    );

    let window = wrapper.window;

    begin_rendering(event_loop, window, wm);
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
            tex_coords: vert.tex_coords,
            lightmap_coords: [0.0, 0.0],
            normal: vert.normal,
            color: [1.0, 1.0, 1.0, 1.0],
            tangent: [0.0, 0.0, 0.0, 0.0],
            uv_offset: vert.animation_uv_offset,
        }
    }

    fn name(&self) -> &str {
        "all"
    }
}

fn begin_rendering(event_loop: EventLoop<()>, window: Window, wm: WmRenderer) {
    let (_entity, mut instances) = describe_entity(&wm);

    wm.pipelines
        .load_full()
        .chunk_layers
        .store(Arc::new(vec![Box::new(TerrainLayer)]));

    let chunk = make_chunks(&wm);

    {
        wm.mc
            .chunks
            .loaded_chunks
            .write()
            .insert([0, 0], ArcSwap::new(Arc::new(chunk)));
    }

    let mut frame_start = Instant::now();

    let mut forward = 0.0;

    let mut spin: f32 = 0.0;

    let mut _frame: u32 = 0;

    let pack = serde_yaml::from_str(
        &wm.mc
            .resource_provider
            .get_string(&ResourcePath("wgpu_mc:graph.yaml".into()))
            .unwrap(),
    )
    .unwrap();
    let mut graph = ShaderGraph::new(pack);

    graph.init(&wm);

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
                        wm.resize(WindowSize {
                            width: physical_size.width,
                            height: physical_size.height,
                        });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        wm.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height,
                        });
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                wm.upload_camera();

                let frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                spin += 0.5;
                _frame += 1;

                let mut camera = **wm.mc.camera.load();

                let direction = camera.get_direction();
                camera.position += direction * 200.0 * frame_time * forward;

                wm.mc.camera.store(Arc::new(camera));

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

                let _ = wm.render(&graph, &view);

                texture.present();

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
