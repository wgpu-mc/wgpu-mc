/*!
# wgpu-mc
wgpu-mc is a pure-Rust crate which is designed to be usable by anyone who needs to render
Minecraft-style scenes using Rust. The main user of this crate at this time is the Minecraft mod
Electrum which replaces Minecraft's official renderer with wgpu-mc.
However, anyone is able to use this crate, and the API is designed to be completely independent
of any single project, allowing anyone to use it. It is mostly batteries-included, except for a
few things.

# Considerations

This crate is unstable and subject to change. The basic structure for features such
as terrain rendering and entity rendering are already in-place but could very well change significantly
in the future.

# Setup
wgpu-mc, as you could have probably guessed, uses the [wgpu](https://github.com/gfx-rs/wgpu) crate
for communicating with the GPU. Assuming you aren't running wgpu-mc headless (if you are, I assume
you already know what you're doing), wgpu-mc can handle surface and device setup for you, as long
as you pass in a valid window handle. See [init_wgpu]

# Rendering

wgpu-mc makes use of a trait called `WmPipeline` to describe any struct which is used for
rendering. There are multiple built in pipelines, but they aren't required to use while rendering.

## Terrain Rendering

The first step to begin terrain rendering is to implement [BlockStateProvider](cr).
This is a trait that provides a block state key for a given coordinate.

## Entity Rendering

To render entities, you need an entity model. wgpu-mc makes no assumptions about how entity models are defined,
so it's up to you to provide them to wgpu-mc.

See the [render::entity] module for an example of rendering an example entity.
 */

use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;

use arc_swap::ArcSwap;
pub use minecraft_assets;
pub use naga;
use parking_lot::RwLock;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::{span, Level};
pub use wgpu;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BufferDescriptor, CompositeAlphaMode, RenderPassDescriptor,
};

use crate::camera::UniformMatrixHelper;
use crate::mc::resource::ResourceProvider;
use crate::mc::MinecraftState;
use crate::render::atlas::Atlas;
use crate::render::pipeline::{RenderPipelineManager, WmPipeline};
use crate::texture::TextureSamplerView;
use crate::util::WmArena;

pub mod camera;
pub mod mc;
pub mod render;
pub mod texture;
pub mod util;

pub struct WgpuState {
    pub surface: RwLock<(Option<wgpu::Surface>, wgpu::SurfaceConfiguration)>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: Option<ArcSwap<WindowSize>>,
}

///The main wgpu-mc renderer struct. This mostly just contains wgpu state.
///Resources pertaining to Minecraft go in `MinecraftState`
#[derive(Clone)]
pub struct WmRenderer {
    pub wgpu_state: Arc<WgpuState>,
    pub depth_texture: Arc<ArcSwap<TextureSamplerView>>,
    pub render_pipeline_manager: Arc<ArcSwap<RenderPipelineManager>>,
    pub mc: Arc<MinecraftState>,
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
    ///This is a convenience method;
    ///
    /// This takes in a raw window handle and returns a [WgpuState], which is then used to
    /// initialize a [WmRenderer].
    pub async fn init_wgpu<W: HasRawWindowHandle + HasRawDisplayHandle + HasWindowSize>(
        window: &W,
        vsync: bool,
    ) -> WgpuState {
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
            present_mode: if vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            // TODO: implement vsync setting
            // if vsync { wgpu::PresentMode::AutoVsync } else { wgpu::PresentMode::AutoNoVsync },
            alpha_mode: CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &surface_config);

        WgpuState {
            surface: RwLock::new((Some(surface), surface_config)),
            adapter,
            device,
            queue,
            size: Some(ArcSwap::new(Arc::new(size))),
        }
    }

    pub fn new(wgpu_state: WgpuState, resource_provider: Arc<dyn ResourceProvider>) -> WmRenderer {
        let pipelines = RenderPipelineManager::new(resource_provider.clone());

        let mc = MinecraftState::new(resource_provider);
        let surface_config = wgpu_state.surface.read().1.clone();

        let depth_texture = TextureSamplerView::create_depth_texture(
            &wgpu_state.device,
            wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
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
                            false,
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

        let surface_state = self.wgpu_state.surface.write(); //Guarantee the Surface is not in use

        let mut surface_config = surface_state.1.clone();

        surface_config.width = new_size.width;
        surface_config.height = new_size.height;

        surface_state
            .0
            .as_ref()
            .unwrap()
            .configure(&self.wgpu_state.device, &surface_config);

        let mut new_camera = *self.mc.camera.load_full();

        new_camera.aspect = surface_config.width as f32 / surface_config.height as f32;
        self.mc.camera.store(Arc::new(new_camera));

        self.depth_texture
            .store(Arc::new(TextureSamplerView::create_depth_texture(
                &self.wgpu_state.device,
                wgpu::Extent3d {
                    width: surface_config.width,
                    height: surface_config.height,
                    depth_or_array_layers: 1,
                },
                "depth_texture",
            )));
    }

    pub fn upload_camera(&self) {
        let mut camera = **self.mc.camera.load();
        let surface_config = &self.wgpu_state.surface.read().1;
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

    pub fn upload_animated_block_buffer(&self, data: Vec<f32>) {
        let d = data.as_slice();

        let buf = self.mc.animated_block_buffer.borrow().load_full();

        if buf.is_none() {
            let animated_block_buffer = self.wgpu_state.device.create_buffer(&BufferDescriptor {
                label: None,
                size: (d.len() * 8) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let animated_block_bind_group =
                self.wgpu_state
                    .device
                    .create_bind_group(&BindGroupDescriptor {
                        label: None,
                        layout: self
                            .render_pipeline_manager
                            .load()
                            .bind_group_layouts
                            .read()
                            .get("ssbo")
                            .unwrap(),
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                animated_block_buffer.as_entire_buffer_binding(),
                            ),
                        }],
                    });

            self.mc
                .animated_block_buffer
                .store(Arc::new(Some(animated_block_buffer)));
            self.mc
                .animated_block_bind_group
                .store(Arc::new(Some(animated_block_bind_group)));
        }

        self.wgpu_state.queue.write_buffer(
            (**self.mc.animated_block_buffer.load()).as_ref().unwrap(),
            0,
            bytemuck::cast_slice(d),
        );
    }

    pub fn update_animated_textures(&self, _subframe: u32) {
        // self.upload_animated_block_buffer(
        //     self
        //     .mc
        //     .texture_manager.atlases
        //     .load_full()
        //     .get(BLOCK_ATLAS_NAME)
        //     .unwrap().load_full().update_textures(subframe)
        // );
    }

    pub fn render(
        &self,
        wm_pipelines: &[&dyn WmPipeline],
        output_texture_view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        let _span_ = span!(Level::TRACE, "render").entered();

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
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_texture_view,
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
                })],
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

        Ok(())
    }

    pub fn get_backend_description(&self) -> String {
        format!(
            "wgpu 0.14 ({:?})",
            self.wgpu_state.adapter.get_info().backend
        )
    }
}
