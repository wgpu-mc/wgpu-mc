#![feature(set_ptr_value)]

use std::iter;
use tracing::{span, Level};

pub mod camera;
pub mod mc;
pub mod model;
pub mod render;
pub mod texture;
pub mod util;

pub use naga;
pub use wgpu;

use crate::camera::UniformMatrixHelper;

use crate::mc::MinecraftState;

use raw_window_handle::HasRawWindowHandle;

use std::collections::HashMap;
use wgpu::{RenderPassDescriptor, TextureFormat, TextureViewDescriptor};

use crate::texture::TextureSamplerView;

use std::sync::Arc;

use crate::mc::resource::ResourceProvider;

use crate::render::pipeline::{RenderPipelineManager, WmPipeline};
use arc_swap::ArcSwap;

use crate::render::atlas::Atlas;

use crate::util::WmArena;

pub struct WgpuState {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: ArcSwap<wgpu::SurfaceConfiguration>,
    pub size: ArcSwap<WindowSize>,
}

///Data specific to wgpu and rendering goes here, everything specific to Minecraft
/// goes in `MinecraftState`
#[derive(Clone)]
pub struct WmRenderer {
    pub wgpu_state: Arc<WgpuState>,

    pub depth_texture: Arc<ArcSwap<texture::TextureSamplerView>>,

    pub render_pipeline_manager: Arc<ArcSwap<RenderPipelineManager>>,

    pub mc: Arc<mc::MinecraftState>,
}

#[derive(Copy, Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub trait HasWindowSize {
    fn get_window_size(&self) -> WindowSize;
}

impl WmRenderer {
    pub async fn init_wgpu<W: HasRawWindowHandle + HasWindowSize>(window: &W) -> WgpuState {
        let size = window.get_window_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default() | wgpu::Features::DEPTH_CLIP_CONTROL,
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &surface_config);

        WgpuState {
            surface,
            adapter,
            device,
            queue,
            surface_config: ArcSwap::new(Arc::new(surface_config)),
            size: ArcSwap::new(Arc::new(size)),
        }
    }

    pub fn new(wgpu_state: WgpuState, resource_provider: Arc<dyn ResourceProvider>) -> WmRenderer {
        let pipelines = render::pipeline::RenderPipelineManager::new(resource_provider.clone());

        let mc = MinecraftState::new(&wgpu_state, &pipelines, resource_provider);
        let depth_texture = TextureSamplerView::create_depth_texture(
            &wgpu_state.device,
            &wgpu_state.surface_config.load(),
            "depth texture",
        );

        Self {
            wgpu_state: Arc::new(wgpu_state),

            depth_texture: Arc::new(ArcSwap::new(Arc::new(depth_texture))),
            render_pipeline_manager: Arc::new(ArcSwap::new(Arc::new(pipelines))),
            mc: Arc::new(mc),
        }
    }

    pub fn init(&self, pipelines: &[&dyn WmPipeline]) {
        self.init_pipeline_manager(pipelines);

        let pipeline_manager = self.render_pipeline_manager.load();

        let atlas_map: HashMap<_, _> = pipelines
            .iter()
            .flat_map(|&pipeline| {
                let atlases = pipeline.atlases();

                atlases.iter().map(|&atlas_name| {
                    (
                        String::from(atlas_name),
                        Arc::new(ArcSwap::new(Arc::new(Atlas::new(
                            &self.wgpu_state,
                            &pipeline_manager,
                        )))),
                    )
                })
            })
            .collect();

        self.mc.texture_manager.atlases.store(Arc::new(atlas_map));

        self.init_mc();
    }

    fn init_pipeline_manager(&self, pipelines: &[&dyn WmPipeline]) {
        self.render_pipeline_manager.load().init(self, pipelines);
    }

    fn init_mc(&self) {
        self.mc.init_camera(self);
    }

    pub fn resize(&self, new_size: WindowSize) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        let mut surface_config = (*self.wgpu_state.surface_config.load_full()).clone();

        surface_config.width = new_size.width;
        surface_config.height = new_size.height;

        self.wgpu_state
            .surface
            .configure(&self.wgpu_state.device, &surface_config);

        let mut new_camera = *self.mc.camera.load_full();

        new_camera.aspect = surface_config.width as f32 / surface_config.height as f32;
        self.mc.camera.store(Arc::new(new_camera));

        self.depth_texture
            .store(Arc::new(texture::TextureSamplerView::create_depth_texture(
                &self.wgpu_state.device,
                &surface_config,
                "depth_texture",
            )));
    }

    pub fn update(&self) {
        // self.camera_controller.update_camera(&mut self.camera);
        // self.mc.camera.update_view_proj(&self.camera);
        let mut camera = **self.mc.camera.load();
        let surface_config = self.wgpu_state.surface_config.load();
        camera.aspect = surface_config.width as f32 / surface_config.height as f32;

        let uniforms = UniformMatrixHelper {
            view_proj: camera.build_view_projection_matrix().into(),
        };

        self.mc.camera.store(Arc::new(camera));

        self.wgpu_state.queue.write_buffer(
            (*self.mc.camera_buffer.load_full()).as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    pub fn render(&self, wm_pipelines: &[&dyn WmPipeline]) -> Result<(), wgpu::SurfaceError> {
        let _span_ = span!(Level::TRACE, "rendering").entered();

        let output = self.wgpu_state.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder =
            self.wgpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let depth_texture = self.depth_texture.load();
        let mut arena = WmArena::new(8000);

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
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
        format!(
            "Wgpu 0.12 ({:?})",
            self.wgpu_state.adapter.get_info().backend
        )
    }
}
