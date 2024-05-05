use winit::raw_window_handle::{HasDisplayHandle, HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use arc_swap::ArcSwap;

use futures::executor::block_on;
use parking_lot::RwLock;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use wgpu_mc::{HasWindowSize, wgpu, WgpuState, WindowSize, WmRenderer};
use wgpu_mc::mc::block::{BlockMeshVertex, BlockstateKey};
use wgpu_mc::mc::chunk::{LightLevel, RenderLayer};
use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::mc::Scene;
use wgpu_mc::render::graph::{RenderGraph, ResourceBacking};
use wgpu_mc::render::pipeline::Vertex;
use wgpu_mc::render::shaderpack::ShaderPackConfig;
use wgpu_mc::wgpu::{BufferBindingType, Extent3d, PresentMode};
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};


use crate::camera::Camera;
use crate::chunk::make_chunks;

mod camera;
mod chunk;

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


fn main() {
    let event_loop = EventLoop::new().unwrap();
    let title = "wgpu-mc test";
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap()
    );

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

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(window.clone()) }.unwrap();
    let adapter = block_on(instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

    let required_limits = wgpu::Limits {
        max_push_constant_size: 128,
        max_bind_groups: 8,
        ..Default::default()
    };

    let (device, queue) = block_on(adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default()
                    | wgpu::Features::DEPTH_CLIP_CONTROL
                    | wgpu::Features::PUSH_CONSTANTS,
                required_limits,
            },
            None, // Trace path
        ))
        .unwrap();

    const VSYNC: bool = false;

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: if VSYNC {
            PresentMode::AutoVsync
        } else if surface_caps.present_modes.contains(&PresentMode::Immediate) {
            PresentMode::Immediate
        } else {
            surface_caps.present_modes[0]
        },

        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &surface_config);

    let wgpu_state = WgpuState {
        surface: RwLock::new((Some(surface), surface_config)),
        adapter,
        device,
        queue,
        size: Some(ArcSwap::new(Arc::new(WindowSize {
            width: window.inner_size().width,
            height: window.inner_size().height,
        }))),
    };

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

    wm.mc.bake_blocks(&wm, blocks.iter().map(|(a, b)| (a, b)));

    begin_rendering(event_loop, window, wm);
}

pub struct TerrainLayer;

fn create_buffer(wm: &WmRenderer, contents: &[u8]) -> wgpu::Buffer {
    wm.wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn begin_rendering(event_loop: EventLoop<()>, window: Arc<Window>, wm: WmRenderer) {
    let pack = serde_yaml::from_str::<ShaderPackConfig>(
        &wm.mc
            .resource_provider
            .get_string(&ResourcePath("wgpu_mc:graph.yaml".into()))
            .unwrap(),
    );

    let mat4_model_buffer = Arc::new(create_buffer(&wm, &[0; 64]));
    let mat4_view_buffer = Arc::new(create_buffer(&wm, &[0; 64]));
    let mat4_persp_buffer = Arc::new(create_buffer(&wm, &[0; 64]));

    let resource_backings = [
        ("@mat4_model".into(), ResourceBacking::BufferBacked(mat4_model_buffer.clone(), BufferBindingType::Uniform)),
        ("@mat4_view".into(), ResourceBacking::BufferBacked(mat4_view_buffer.clone(), BufferBindingType::Uniform)),
        ("@mat4_perspective".into(), ResourceBacking::BufferBacked(mat4_persp_buffer.clone(), BufferBindingType::Uniform))
    ].into_iter().collect::<HashMap<String, ResourceBacking>>();

    let render_graph = RenderGraph::new(&wm, resource_backings, pack.unwrap());

    let scene = Scene::new(&wm, Extent3d {
        width: window.inner_size().width,
        height: window.inner_size().height,
        depth_or_array_layers: 1,
    });

    let section = make_chunks(&wm);
    scene.chunk_sections.insert([0,0,0].into(), section);


    let mut forward = 0.0;

    let mut camera = Camera::new(window.inner_size().width as f32 / window.inner_size().height as f32);

    event_loop
        .run(move |event, target| {
            match event {
                Event::AboutToWait => window.request_redraw(),
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::KeyboardInput { event, .. } => match event {
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Space),
                                ..
                            } => {
                                //Update a block and re-generate the chunk mesh for testing

                                //removed atm
                            }
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            } => target.exit(),
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::KeyW),
                                ..
                            } => {
                                forward = 1.0;
                            }
                            KeyEvent {
                                state: ElementState::Released,
                                physical_key: PhysicalKey::Code(KeyCode::KeyW),
                                ..
                            } => {
                                forward = 0.0;
                            }
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::KeyS),
                                ..
                            } => {
                                forward = -1.0;
                            }
                            KeyEvent {
                                state: ElementState::Released,
                                physical_key: PhysicalKey::Code(KeyCode::KeyS),
                                ..
                            } => {
                                forward = 0.0;
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            let state_size = wm.wgpu_state.size.as_ref().unwrap();
                            state_size.swap(Arc::new(WindowSize {
                                width: physical_size.width,
                                height: physical_size.height,
                            }));
                        }
                        WindowEvent::RedrawRequested => {

                            let perspective: [[f32; 4]; 4] = camera.build_perspective_matrix().into();
                            let view: [[f32; 4]; 4] = camera.build_view_matrix().into();

                            wm.wgpu_state.queue.write_buffer(
                                &mat4_persp_buffer,
                                0,
                                bytemuck::cast_slice(&perspective)
                            );

                            wm.wgpu_state.queue.write_buffer(
                                &mat4_view_buffer,
                                0,
                                bytemuck::cast_slice(&view)
                            );

                            camera.position += camera.get_direction() * forward * 0.01;

                            let mut surface_guard = wm.wgpu_state.surface.write();
                            let (surface, ref mut config) = &mut *surface_guard;

                            let surface = surface.as_ref().unwrap();

                            let surface_texture = surface.get_current_texture().unwrap_or_else(|_| {
                                //The surface is outdated, so we force an update. This can't be done on the window resize event for synchronization reasons.
                                let size = wm.wgpu_state.size.as_ref().unwrap().load();

                                config.width = size.width;
                                config.height = size.height;

                                surface.configure(&wm.wgpu_state.device, &config);
                                surface.get_current_texture().unwrap()
                            });

                            let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
                                label: None,
                                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                                dimension: Some(wgpu::TextureViewDimension::D2),
                                aspect: Default::default(),
                                base_mip_level: 0,
                                mip_level_count: None,
                                base_array_layer: 0,
                                array_layer_count: None,
                            });


                            wm.submit_chunk_updates();
                            render_graph.render(&wm, &scene, &view, [0; 3]);

                            surface_texture.present();

                        }
                        _ => {}
                    }
                }
                // Event::DeviceEvent { event: DeviceEvent::Added {..}, ..} => {}
                // Event::DeviceEvent { event: DeviceEvent::Removed {..}, ..} => {}
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    camera.yaw += (delta.0 / 100.0) as f32;
                    camera.pitch -= (delta.1 / 100.0) as f32;
                }
                _ => {}
            }
        })
        .unwrap();
}
