use std::iter;

use wgpu::util::DeviceExt;

pub mod mc;

pub mod camera;
pub mod model;
pub mod texture;
mod render;

use crate::camera::{Camera, CameraController, Uniforms};
use crate::mc::chunk::Chunk;
use crate::mc::MinecraftRenderer;

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
    fn get_shader(&self, name: &str) -> &str;
}

pub struct MinecraftRegistry {
    pub items: HashSet<String>,
    pub blocks: HashSet<String>
}

impl MinecraftRegistry {
    pub fn new() -> Self {
        Self {
            items: Default::default(),
            blocks: Default::default()
        }
    }
}

///Data specific to wgpu and rendering goes here, everything specific to Minecraft and it's state
/// goes in MinecraftRenderer
pub struct Renderer {
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub size: Arc<RwLock<WindowSize>>,

    pub depth_texture: texture::WgpuTexture,

    pub pipelines: RenderPipelinesManager,

    pub mc: mc::MinecraftRenderer,

    pub registry: MinecraftRegistry
}

#[derive(Copy, Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32
}

pub trait HasWindowSize {
    fn get_window_size(&self) -> WindowSize;
}

impl Renderer{
    pub async fn new<W: HasRawWindowHandle + HasWindowSize>(
        window: &W,
        resource_provider: Arc<dyn ResourceProvider>,
        shader_provider: Arc<dyn ShaderProvider>
    ) -> Renderer {
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

        let mc = MinecraftRenderer::new(&device, &pipelines, resource_provider, shader_provider);
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
            mc,

            registry: MinecraftRegistry::new()
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

    pub fn render(&mut self, chunks: &[Chunk]) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None
            });


        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let atlases = self.mc.texture_manager.atlases.read();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.pipelines.terrain_pipeline);

            //Render chunks

            let texture_bind_group = &atlases.block.material.as_ref().unwrap().bind_group;

            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.mc.uniform_bind_group, &[]);

            let mrp = &mut render_pass;

            for chunk in chunks.iter() {
                render_pass.set_vertex_buffer(
                    0,
                    match &chunk.vertex_buffer {
                        None => panic!("Chunk did not have generated vertex buffer!"),
                        Some(buf) => buf.slice(..),
                    },
                );

                render_pass.draw(0..chunk.vertex_count as u32, 0..1);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn get_backend_description(&self) -> String {
        format!("Wgpu 11.0 ({:?})", self.adapter.get_info().backend)
    }
}
