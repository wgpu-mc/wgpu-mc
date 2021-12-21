#![feature(set_ptr_value)]

use std::iter;

use wgpu::util::DeviceExt;

pub mod mc;
pub mod camera;
pub mod model;
pub mod texture;
pub mod render;
pub mod util;

use crate::camera::{Camera, UniformMatrixHelper};
use crate::mc::chunk::Chunk;
use crate::mc::MinecraftState;

use raw_window_handle::HasRawWindowHandle;
use shaderc::ShaderKind;
use winit::event::WindowEvent;
use wgpu::{RenderPass, VertexState, TextureViewDescriptor, RenderPassDescriptor};
use std::collections::{HashMap, HashSet};
use crate::render::shader::{Shader, ShaderSource};
use crate::texture::WgpuTexture;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::{RwLock, Mutex};
use dashmap::DashMap;
use crate::mc::resource::ResourceProvider;
use std::ops::{DerefMut, Deref};
use std::cell::RefCell;
use crate::render::pipeline::{RenderPipelinesManager, WmPipeline};
use arc_swap::ArcSwap;
use crate::util::WmArena;

macro_rules! dashmap(
    { $($key:expr => $value:expr),+ } => {
        {
            let m = dashmap::DashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct WgpuState {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue
}

///Data specific to wgpu and rendering goes here, everything specific to Minecraft and it's state
/// goes in `MinecraftState`
pub struct WmRenderer {
    pub wgpu_state: Arc<WgpuState>,

    pub surface_config: ArcSwap<wgpu::SurfaceConfiguration>,

    pub size: ArcSwap<WindowSize>,

    pub depth_texture: ArcSwap<texture::WgpuTexture>,

    pub pipelines: ArcSwap<RenderPipelinesManager>,

    pub mc: Arc<mc::MinecraftState>,
    
    pub shaderc: Arc<Mutex<shaderc::Compiler>>
}

#[derive(Copy, Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32
}

pub trait HasWindowSize {
    fn get_window_size(&self) -> WindowSize;
}

impl WmRenderer {
    pub async fn new<W: HasRawWindowHandle + HasWindowSize>(
        window: &W,
        resource_provider: Arc<dyn ResourceProvider>
    ) -> WmRenderer {
        let size = window.get_window_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default()
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let mut sc = shaderc::Compiler::new().unwrap();

        let shader_map = dashmap! {
            String::from("sky") => Shader::from_glsl(ShaderSource {
                file_name: "sky.fsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/sky.fsh").into())).unwrap(),
                entry_point: "main"
            }, ShaderSource {
                file_name: "sky.vsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/sky.vsh").into())).unwrap(),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("terrain") => Shader::from_glsl(ShaderSource {
                file_name: "terrain.fsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/terrain.fsh").into())).unwrap(),
                entry_point: "main"
            }, ShaderSource {
                file_name: "terrain.vsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/terrain.vsh").into())).unwrap(),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("grass") => Shader::from_glsl(ShaderSource {
                file_name: "grass.fsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/grass.fsh").into())).unwrap(),
                entry_point: "main"
            }, ShaderSource {
                file_name: "grass.vsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/grass.vsh").into())).unwrap(),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("transparent") => Shader::from_glsl(ShaderSource {
                file_name: "transparent.fsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/transparent.fsh").into())).unwrap(),
                entry_point: "main"
            }, ShaderSource {
                file_name: "transparent.vsh",
                source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/transparent.vsh").into())).unwrap(),
                entry_point: "main"
            }, &device, &mut sc).unwrap()

        };
        
        let pipelines = render::pipeline::RenderPipelinesManager::init(
            &device,
            shader_map,
            resource_provider.clone());

        let mc = MinecraftState::new(&device, &pipelines, resource_provider);
        let depth_texture = WgpuTexture::create_depth_texture(&device, &config, "depth texture");

        let wgpu_state = WgpuState {
            surface,
            adapter,
            device,
            queue
        };

        Self {
            wgpu_state: Arc::new(wgpu_state),
            surface_config: ArcSwap::new(Arc::new(config)),
            size: ArcSwap::new(Arc::new(size)),

            depth_texture: ArcSwap::new(Arc::new(depth_texture)),
            pipelines: ArcSwap::new(Arc::new(pipelines)),
            mc: Arc::new(mc),
            shaderc: Arc::new(Mutex::new(sc))
        }
    }

    pub fn resize(&self, new_size: WindowSize) {
        let mut surface_config = (*self.surface_config.load_full()).clone();

        surface_config.width = new_size.width;
        surface_config.height = new_size.height;

        self.wgpu_state.surface.configure(&self.wgpu_state.device, &surface_config);

        let mut new_camera = *self.mc.camera.load_full().clone();

        new_camera.aspect = surface_config.height as f32 / surface_config.width as f32;
        self.mc.camera.store(Arc::new(new_camera));

        self.depth_texture.store(Arc::new(texture::WgpuTexture::create_depth_texture(&self.wgpu_state.device, &surface_config, "depth_texture")));
    }

    pub fn update(&mut self) {
        // self.camera_controller.update_camera(&mut self.camera);
        // self.mc.camera.update_view_proj(&self.camera);
        let mut camera = **self.mc.camera.load();
        let surface_config = self.surface_config.load();
        camera.aspect = surface_config.height as f32 / surface_config.width as f32;

        let uniforms = UniformMatrixHelper {
            view_proj: camera.build_view_projection_matrix().into()
        };

        self.mc.camera.store(Arc::new(camera));

        self.wgpu_state.queue.write_buffer(
            &self.mc.uniform_buffer.load_full(),
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    pub fn render(&self, wm_pipelines: &[&dyn WmPipeline]) -> Result<(), wgpu::SurfaceError> {
        let output = self.wgpu_state.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .wgpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let depth_texture = self.depth_texture.load();
        let mut arena = WmArena::new(8000);

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0
                            }),
                            store: true
                        }
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });

            for &wm_pipeline in wm_pipelines {
                wm_pipeline.render(self, &mut render_pass, &mut arena);
            }

        }
        self.wgpu_state.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn get_backend_description(&self) -> String {
        format!("Wgpu 0.12 ({:?})", self.wgpu_state.adapter.get_info().backend)
    }

}
