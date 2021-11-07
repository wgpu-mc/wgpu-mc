use std::iter;

use wgpu::util::DeviceExt;

pub mod mc;
pub mod camera;
pub mod model;
pub mod texture;
pub mod render;

use crate::camera::{Camera, CameraController, Uniforms};
use crate::mc::chunk::Chunk;
use crate::mc::MinecraftState;

use raw_window_handle::HasRawWindowHandle;
use shaderc::ShaderKind;
use winit::event::WindowEvent;
use wgpu::{RenderPass, VertexState, TextureViewDescriptor};
use std::collections::{HashMap, HashSet};
use crate::render::pipeline::{RenderPipelinesManager};
use crate::render::shader::{Shader, ShaderSource};
use crate::texture::WgpuTexture;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use crate::mc::resource::ResourceProvider;

macro_rules! dashmap(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = dashmap::DashMap::new();
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

pub trait ShaderProvider: Send + Sync {
    fn get_shader(&self, name: &str) -> String;
}

///Data specific to wgpu and rendering goes here, everything specific to Minecraft and it's state
/// goes in MinecraftState
pub struct WmRenderer {
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub size: Arc<RwLock<WindowSize>>,

    pub depth_texture: texture::WgpuTexture,

    pub pipelines: RenderPipelinesManager,

    pub mc: mc::MinecraftState
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
        resource_provider: Arc<dyn ResourceProvider>,
        shader_provider: Arc<dyn ShaderProvider>
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
                source: &shader_provider.get_shader("sky.fsh"),
                entry_point: "main"
            }, ShaderSource {
                file_name: "sky.vsh",
                source: &shader_provider.get_shader("sky.vsh"),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("terrain") => Shader::from_glsl(ShaderSource {
                file_name: "terrain.fsh",
                source: &shader_provider.get_shader("terrain.fsh"),
                entry_point: "main"
            }, ShaderSource {
                file_name: "terrain.vsh",
                source: &shader_provider.get_shader("terrain.vsh"),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("grass") => Shader::from_glsl(ShaderSource {
                file_name: "grass.fsh",
                source: &shader_provider.get_shader("grass.fsh"),
                entry_point: "main"
            }, ShaderSource {
                file_name: "grass.vsh",
                source: &shader_provider.get_shader("grass.vsh"),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("transparent") => Shader::from_glsl(ShaderSource {
                file_name: "transparent.fsh",
                source: &shader_provider.get_shader("transparent.fsh"),
                entry_point: "main"
            }, ShaderSource {
                file_name: "transparent.vsh",
                source: &shader_provider.get_shader("transparent.vsh"),
                entry_point: "main"
            }, &device, &mut sc).unwrap(),

            String::from("gui") => Shader::from_glsl(ShaderSource {
                file_name: "gui.fsh",
                source: &shader_provider.get_shader("gui.fsh"),
                entry_point: "main"
            }, ShaderSource {
                file_name: "gui.vsh",
                source: &shader_provider.get_shader("gui.vsh"),
                entry_point: "main"
            }, &device, &mut sc).unwrap()
        };
        
        let pipelines = render::pipeline::RenderPipelinesManager::init(
            &device,
            shader_map,
            shader_provider.clone());

        let mc = MinecraftState::new(&device, &pipelines, resource_provider, shader_provider);
        let depth_texture = WgpuTexture::create_depth_texture(&device, &config, "depth texture");

        Self {
            surface,
            surface_config: config,

            adapter,

            device,
            queue,
            size: Arc::new(RwLock::new(size)),

            depth_texture,
            pipelines,
            mc
        }
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;

        self.surface.configure(&self.device, &self.surface_config);

        self.mc.camera.aspect = self.surface_config.height as f32 / self.surface_config.width as f32;
        // self.size = new_size;
        self.depth_texture =
            texture::WgpuTexture::create_depth_texture(&self.device, &self.surface_config, "depth_texture");
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // se
        //lf.camera_controller.process_events(event)
        false
    }

    pub fn update(&mut self) {
        // self.camera_controller.update_camera(&mut self.camera);
        // self.mc.camera.update_view_proj(&self.camera);
        self.mc.camera.aspect = self.surface_config.height as f32 / self.surface_config.width as f32;

        let uniforms = Uniforms {
            view_proj: self.mc.camera.build_view_projection_matrix().into()
        };

        self.queue.write_buffer(
            &self.mc.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.pipelines.pipelines.read().iter().for_each(|wm_pipeline| {
            wm_pipeline.render(
                &self,
                &output,
                &view,
                &encoder,

            );
        });

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn get_backend_description(&self) -> String {
        format!("Wgpu 11.0 ({:?})", self.adapter.get_info().backend)
    }
}
