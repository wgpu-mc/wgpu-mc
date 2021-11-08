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
use wgpu::{RenderPass, VertexState, TextureViewDescriptor, RenderPassDescriptor};
use std::collections::{HashMap, HashSet};
use crate::render::shader::{Shader, ShaderSource};
use crate::texture::WgpuTexture;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use crate::mc::resource::ResourceProvider;
use std::ops::{DerefMut, Deref};
use std::cell::RefCell;
use crate::render::pipeline::{RenderPipelinesManager, WmPipeline};

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

pub struct WgpuState {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue
}

///Data specific to wgpu and rendering goes here, everything specific to Minecraft and it's state
/// goes in `MinecraftState`
#[derive(Clone)]
pub struct WmRenderer {
    pub wgpu_state: Arc<WgpuState>,

    pub surface_config: Arc<RwLock<wgpu::SurfaceConfiguration>>,

    pub size: Arc<RwLock<WindowSize>>,

    pub depth_texture: Arc<RwLock<texture::WgpuTexture>>,

    pub pipelines: Arc<RwLock<RenderPipelinesManager>>,

    pub mc: Arc<mc::MinecraftState>
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

        let wgpu_state = WgpuState {
            surface,
            adapter,
            device,
            queue
        };

        Self {
            wgpu_state: Arc::new(wgpu_state),
            surface_config: Arc::new(RwLock::new(config)),
            size: Arc::new(RwLock::new(size)),

            depth_texture: Arc::new(RwLock::new(depth_texture)),
            pipelines: Arc::new(RwLock::new(pipelines)),
            mc: Arc::new(mc)
        }
    }

    pub fn resize(&self, new_size: WindowSize) {
        let mut surface_config = self.surface_config.write();

        surface_config.width = new_size.width;
        surface_config.height = new_size.height;

        self.wgpu_state.surface.configure(&self.wgpu_state.device, &surface_config);

        self.mc.camera.write().aspect = surface_config.height as f32 / surface_config.width as f32;
        // self.size = new_size;
        *(self.depth_texture.write().deref_mut()) =
            texture::WgpuTexture::create_depth_texture(&self.wgpu_state.device, &surface_config, "depth_texture");
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // se
        //lf.camera_controller.process_events(event)
        false
    }

    pub fn update(&mut self) {
        // self.camera_controller.update_camera(&mut self.camera);
        // self.mc.camera.update_view_proj(&self.camera);
        let mut camera = self.mc.camera.write();
        let surface_config = self.surface_config.read();
        camera.aspect = surface_config.height as f32 / surface_config.width as f32;

        let uniforms = Uniforms {
            view_proj: camera.build_view_projection_matrix().into()
        };

        self.wgpu_state.queue.write_buffer(
            &self.mc.uniform_buffer.read(),
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

        let pipelines = self.pipelines.read();
        let chunks = self.mc.chunks.read();
        let chunk_reads: Vec<_> = chunks.loaded_chunks.iter().map(|c| c.read()).collect();

        let chunk_slice: Vec<&Chunk> = chunk_reads.iter().map(|guard| guard.deref()).collect();

        let depth_texture = self.depth_texture.read();
        let entities = self.mc.entities.read();
        let camera = self.mc.camera.read();
        let uniforms = self.mc.uniform_bind_group.read();
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
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
                render_pass = wm_pipeline.render(self, render_pass, &pipelines, &chunk_slice, &entities, &camera, &uniforms);
            }

        }
        self.wgpu_state.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[must_use]
    pub fn get_backend_description(&self) -> String {
        format!("Wgpu 11.0 ({:?})", self.wgpu_state.adapter.get_info().backend)
    }
}
