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
    BindGroupDescriptor, BindGroupEntry, BufferDescriptor, CompositeAlphaMode, Extent3d,
    RenderPassDescriptor, Texture,
};

use crate::camera::UniformMatrixHelper;
use crate::mc::resource::ResourceProvider;
use crate::mc::MinecraftState;
use crate::render::atlas::Atlas;
use crate::render::graph::ShaderGraph;
use crate::render::pipeline::{WmPipelines, BLOCK_ATLAS, ENTITY_ATLAS};
use crate::render::shaderpack::ShaderPackConfig;
use crate::texture::{BindableTexture, TextureHandle, TextureSamplerView};
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
    pub texture_handles: Arc<RwLock<HashMap<String, TextureHandle>>>,
    pub pipelines: Arc<ArcSwap<WmPipelines>>,
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
        env_logger::init();

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

        let mut limits = wgpu::Limits::default();
        limits.max_push_constant_size = 128;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default()
                        | wgpu::Features::DEPTH_CLIP_CONTROL
                        | wgpu::Features::PUSH_CONSTANTS,
                    limits,
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
        let pipelines = WmPipelines::new(resource_provider.clone());

        let mc = MinecraftState::new(resource_provider);

        Self {
            wgpu_state: Arc::new(wgpu_state),

            texture_handles: Arc::new(RwLock::new(HashMap::new())),
            pipelines: Arc::new(ArcSwap::new(Arc::new(pipelines))),
            mc: Arc::new(mc),
        }
    }

    pub fn init(&self) {
        let pipelines = self.pipelines.load();
        pipelines.init(self);

        let atlases = [BLOCK_ATLAS, ENTITY_ATLAS]
            .iter()
            .map(|&name| {
                (
                    name.into(),
                    Arc::new(ArcSwap::new(Arc::new(Atlas::new(
                        &self.wgpu_state,
                        &pipelines,
                        false,
                    )))),
                )
            })
            .collect();

        self.mc.texture_manager.atlases.store(Arc::new(atlases));
        self.mc.init_camera(self);

        self.create_texture_handle(
            "wm_framebuffer_depth".into(),
            wgpu::TextureFormat::Depth32Float,
        );
    }

    pub fn create_texture_handle(
        &self,
        name: String,
        format: wgpu::TextureFormat,
    ) -> TextureHandle {
        let surface = self.wgpu_state.surface.read();
        let config = &surface.1;

        let tsv = TextureSamplerView::from_rgb_bytes(
            &self.wgpu_state,
            &[],
            Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            None,
            format,
        )
        .unwrap();

        let handle = TextureHandle {
            bindable_texture: Arc::new(ArcSwap::new(Arc::new(BindableTexture::from_tsv(
                &self.wgpu_state,
                &**self.pipelines.load(),
                tsv,
                matches!(format, wgpu::TextureFormat::Depth32Float),
            )))),
        };

        let mut handles = self.texture_handles.write();
        if handles.contains_key(&name) {
            handles
                .get(&name)
                .unwrap()
                .bindable_texture
                .store(handle.bindable_texture.load_full());
        } else {
            handles.insert(name, handle.clone());
        }

        handle
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

        let handles = { self.texture_handles.read().clone() };

        drop(surface_state);

        handles.iter().for_each(|(name, handle)| {
            let texture = handle.bindable_texture.load();

            self.create_texture_handle(name.clone(), texture.tsv.format);
        });
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
                            .pipelines
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

    pub fn render(
        &self,
        graph: &ShaderGraph,
        output_texture_view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        graph.render(self, output_texture_view);

        Ok(())
    }

    pub fn get_backend_description(&self) -> String {
        format!(
            "wgpu 0.14 ({:?})",
            self.wgpu_state.adapter.get_info().backend
        )
    }
}
