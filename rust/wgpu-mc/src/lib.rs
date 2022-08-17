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
use std::iter;
use tracing::{span, Level};

pub mod camera;
pub mod mc;
pub mod render;
pub mod texture;
pub mod util;

pub use minecraft_assets;
pub use naga;
pub use wgpu;

use crate::camera::UniformMatrixHelper;

use crate::mc::MinecraftState;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::collections::HashMap;
use wgpu::{
    Adapter, Backends, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferDescriptor,
    Device, Features, PowerPreference, Queue, RenderPassDescriptor, Surface, SurfaceConfiguration,
    SurfaceError, TextureFormat, TextureViewDescriptor,
};

use crate::texture::TextureSamplerView;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::mc::resource::ResourceProvider;

use crate::render::pipeline::{RenderPipelineManager, WmPipeline};
use arc_swap::ArcSwap;
use parking_lot::RwLock;
use wgpu_biolerless::{DeviceRequirements, State, StateBuilder, WindowSize};

use crate::render::atlas::Atlas;
use crate::render::pipeline::terrain::BLOCK_ATLAS_NAME;

use crate::util::WmArena;

///The main wgpu-mc renderer struct. This mostly just contains wgpu state.
///Resources pertaining to Minecraft go in `MinecraftState`
#[derive(Clone)]
pub struct WmRenderer {
    pub wgpu_state: Arc<State>,

    pub depth_texture: Arc<ArcSwap<TextureSamplerView>>,

    pub render_pipeline_manager: Arc<ArcSwap<RenderPipelineManager>>,

    pub mc: Arc<MinecraftState>,
}

impl WmRenderer {
    ///This is a convenience method;
    ///
    /// This takes in a raw window handle and returns a [State], which is then used to
    /// initialize a [WmRenderer].
    pub async fn init_wgpu<W: WindowSize>(window: W) -> anyhow::Result<State> {
        //Vulkan works just fine, the issue is that using RenderDoc + Vulkan makes it hang on launch
        //about 90% of the time. DX12 is much more stable
        #[cfg(target_os = "windows")]
        let backend = Backends::DX12;
        #[cfg(not(target_os = "windows"))]
        let backend = Backends::PRIMARY;
        StateBuilder::new()
            .window(window)
            .power_pref(PowerPreference::HighPerformance)
            .device_requirements(DeviceRequirements {
                features: Features::default() | Features::DEPTH_CLIP_CONTROL,
                limits: Default::default(),
            })
            .format(TextureFormat::Bgra8Unorm)
            .backends(backend)
            .build()
            .await
    }

    pub fn new(wgpu_state: State, resource_provider: Arc<dyn ResourceProvider>) -> WmRenderer {
        let pipelines = RenderPipelineManager::new(resource_provider.clone());

        let mc = MinecraftState::new(resource_provider);

        let depth_texture = TextureSamplerView::create_depth_texture(&wgpu_state);

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

    pub fn resize(&self, new_size: (u32, u32)) {
        if !self.wgpu_state.resize(new_size) {
            return;
        }

        let mut new_camera = *self.mc.camera.load_full();

        new_camera.aspect = new_size.0 as f32 / new_size.1 as f32;
        self.mc.camera.store(Arc::new(new_camera));

        self.depth_texture
            .store(Arc::new(TextureSamplerView::create_depth_texture(
                &self.wgpu_state,
            )));
    }

    pub fn upload_camera(&self) {
        // self.camera_controller.update_camera(&mut self.camera);
        // self.mc.camera.update_view_proj(&self.camera);
        let mut camera = **self.mc.camera.load();
        let (width, height) = self.wgpu_state.size();
        camera.aspect = width as f32 / height as f32;

        let uniforms = UniformMatrixHelper {
            view_proj: camera.build_view_projection_matrix().into(),
        };

        self.mc.camera.store(Arc::new(camera));

        self.wgpu_state.write_buffer(
            (*self.mc.camera_buffer.load_full()).as_ref().unwrap(),
            0,
            &[uniforms],
        );
    }

    pub fn upload_animated_block_buffer(&self, data: Vec<f32>) {
        let d = data.as_slice();

        let buf = self.mc.animated_block_buffer.borrow().load_full();

        if buf.is_none() {
            let animated_block_buffer = self.wgpu_state.device().create_buffer(&BufferDescriptor {
                label: None,
                size: (d.len() * 8) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let animated_block_bind_group = self.wgpu_state.create_bind_group(
                self.render_pipeline_manager
                    .load()
                    .bind_group_layouts
                    .read()
                    .get("ssbo")
                    .unwrap(),
                &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(
                        animated_block_buffer.as_entire_buffer_binding(),
                    ),
                }],
            );

            self.mc
                .animated_block_buffer
                .store(Arc::new(Some(animated_block_buffer)));
            self.mc
                .animated_block_bind_group
                .store(Arc::new(Some(animated_block_bind_group)));
        }

        self.wgpu_state.write_buffer(
            (**self.mc.animated_block_buffer.load()).as_ref().unwrap(),
            0,
            d,
        );
    }

    pub fn update_animated_textures(&self, subframe: u32) {
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
        output_texture_view: &TextureViewDescriptor,
    ) -> Result<(), SurfaceError> {
        let _span_ = span!(Level::TRACE, "render").entered();

        let depth_texture = self.depth_texture.load();
        let mut arena = WmArena::new(8000);

        self.wgpu_state.render(
            |view, mut encoder, state| {
                {
                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
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
                encoder
            },
            output_texture_view,
        )?;

        Ok(())
    }

    pub fn get_backend_description(&self) -> String {
        format!(
            "wgpu 0.12 ({:?})",
            self.wgpu_state.adapter().get_info().backend
        )
    }
}
