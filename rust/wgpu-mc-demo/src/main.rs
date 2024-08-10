
use arc_swap::ArcSwap;
use parking_lot::lock_api::RwLock;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use futures::executor::block_on;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::mc::Scene;
use wgpu_mc::render::graph::{RenderGraph, ResourceBacking};
use wgpu_mc::render::shaderpack::ShaderPackConfig;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_mc::wgpu::{BufferBindingType, Extent3d, PresentMode};
use wgpu_mc::{wgpu, HasWindowSize, Display, WindowSize, WmRenderer, Frustum};

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
struct Application{
    wm:Option<WmRenderer>,
    forward:f32,
    scene:Option<Scene>,
    render_graph:Option<RenderGraph>,
    camera:Option<Camera>,
    last_frame:Instant
}
impl Application {
    pub fn new()->Self{
        Application{
            wm:None,
            forward:0.0,
            scene:None,
            render_graph:None,
            camera: None,
            last_frame: Instant::now(),
        }
    }
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let title = "wgpu-mc test";

        let window_attributes = 
        winit::window::Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
                width: 1280,
                height: 720,
            }));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let required_limits = wgpu::Limits {
            max_push_constant_size: 128,
            max_bind_groups: 8,
            max_storage_buffers_per_shader_stage: 10000,
            ..Default::default()
        };

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default()
                    | wgpu::Features::DEPTH_CLIP_CONTROL
                    | wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::MULTI_DRAW_INDIRECT,
                required_limits,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None, // Trace path
        ))
        .unwrap();

        const VSYNC: bool = true;

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

        let display = Display {
            surface,
            adapter,
            device,
            queue,
            size: RwLock::new(window.inner_size()),
            window,
            instance,
            config: RwLock::new(surface_config),
        };
    


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
    
    
        let wm = WmRenderer::new(display, rsp);
    
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
        
        wm.init();
    
        wm.mc.bake_blocks(&wm, blocks.iter().map(|(a, b)| (a, b)));

        
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
            (
                "@mat4_model".into(),
                ResourceBacking::Buffer(mat4_model_buffer.clone(), BufferBindingType::Uniform),
            ),
            (
                "@mat4_view".into(),
                ResourceBacking::Buffer(mat4_view_buffer.clone(), BufferBindingType::Uniform),
            ),
            (
                "@mat4_perspective".into(),
                ResourceBacking::Buffer(mat4_persp_buffer.clone(), BufferBindingType::Uniform),
            ),
        ]
        .into_iter()
        .collect::<HashMap<String, ResourceBacking>>();

        self.render_graph = Some(RenderGraph::new(&wm, pack.unwrap(), resource_backings, None, None));

        self.scene = Some(Scene::new(
            &wm,
            Extent3d {
                width: wm.display.window.inner_size().width,
                height: wm.display.window.inner_size().height,
                depth_or_array_layers: 1,
            },
        ));

        {
            let mut sections = self.scene.as_ref().unwrap().chunk_sections.write();

            for x in 0..10 {
                for y in 0..5 {
                    for z in 0..5 {
                        let section = make_chunks(&wm, [x, y, z].into(), self.scene.as_ref().unwrap());

                        sections.insert([x, y, z].into(), RwLock::new(section));
                    }
                }
            }
        }


        self.camera = Some(Camera::new(wm.display.window.inner_size().width as f32 / wm.display.window.inner_size().height as f32));

        self.wm = Some(wm);

    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event{
            DeviceEvent::MouseMotion { delta } => {
                let camera = self.camera.as_mut().unwrap();
                camera.yaw += (delta.0 / 100.0) as f32;
                camera.pitch -= (delta.1 / 100.0) as f32;
            }
            _ => {}
}
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.wm.as_ref().unwrap().display.window.request_redraw()
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
        ) {
        let wm = self.wm.as_ref().unwrap();
        if window_id == wm.display.window.id() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
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
                    } => event_loop.exit(),
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyW),
                        ..
                    } => {
                        self.forward = 1.0;
                    }
                    KeyEvent {
                        state: ElementState::Released,
                        physical_key: PhysicalKey::Code(KeyCode::KeyW),
                        ..
                    } => {
                        self.forward = 0.0;
                    }
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyS),
                        ..
                    } => {
                        self.forward = -1.0;
                    }
                    KeyEvent {
                        state: ElementState::Released,
                        physical_key: PhysicalKey::Code(KeyCode::KeyS),
                        ..
                    } => {
                        self.forward = 0.0;
                    }
                    _ => {}
                },
                WindowEvent::Resized(physical_size) => {
                    *wm.display.size.write() = physical_size;
                }
                WindowEvent::RedrawRequested => {
                    let camera = self.camera.as_mut().unwrap();
                    let wm = self.wm.as_ref().unwrap();
                    let frame_time = Instant::now().duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = Instant::now();

                    let perspective: [[f32; 4]; 4] =
                        camera.build_perspective_matrix().to_cols_array_2d();
                    let view: [[f32; 4]; 4] = camera.build_view_matrix().to_cols_array_2d();

                    if let ResourceBacking::Buffer(buffer,_) = &self.render_graph.as_ref().unwrap().resources["@mat4_perspective"]{
                        wm.display.queue.write_buffer(
                            &buffer,
                            0,
                            bytemuck::cast_slice(&perspective),
                        );
                    }

                    if let ResourceBacking::Buffer(buffer,_) = &self.render_graph.as_ref().unwrap().resources["@mat4_view"]{
                        wm.display.queue.write_buffer(
                            &buffer,
                            0,
                            bytemuck::cast_slice(&view),
                        );
                    }
                    camera.position += camera.get_direction() * self.forward * 50.0 * frame_time;

                    let mut config_guard = wm.display.config.write();

                    let surface_texture =
                        wm.display.surface.get_current_texture().unwrap_or_else(|_| {
                            //The surface is outdated, so we force an update. This can't be done on the window resize event for synchronization reasons.
                            let size = wm.display.size.read();

                            config_guard.width = size.width;
                            config_guard.height = size.height;

                            wm.display.surface.configure(&wm.display.device, &config_guard);
                            wm.display.surface.get_current_texture().unwrap()
                        });

                    let view =
                        surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor {
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

                    let mut command_encoder = wm.display.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: None },
                    );

                    let mut geometry = HashMap::new();

                    let mvp = (camera.build_perspective_matrix() * camera.build_view_matrix()).to_cols_array_2d();

                    self.render_graph.as_ref().unwrap().render(
                        &wm,
                        &mut command_encoder,
                        &self.scene.as_ref().unwrap(),
                        &view,
                        [0; 3],
                        &mut geometry,
                        &Frustum::from_modelview_projection(mvp)
                    );

                    wm.display.queue.submit([command_encoder.finish()]);

                    surface_texture.present();
                }
                _ => {}
            }
        }
    }


}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut application = Application::new();
    event_loop.run_app(&mut application).unwrap();
}

pub struct TerrainLayer;

fn create_buffer(wm: &WmRenderer, contents: &[u8]) -> wgpu::Buffer {
    wm.display
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
}