pub mod transparent;
pub mod grass;
pub mod entity;
pub mod sky;
pub mod terrain;
pub mod debug_lines;

use wgpu::{BindGroupLayout, SamplerBindingType, PipelineLayout};
use crate::render::shader::WmShader;

use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use parking_lot::RwLock;


use crate::{WmRenderer};



use crate::mc::resource::ResourceProvider;


use crate::util::WmArena;



use crate::wgpu::RenderPipeline;

pub trait WmPipeline {

    fn name(&self) -> &'static str;

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>>;

    ///Names of the atlases this pipeline will create
    fn atlases(&self) -> &'static [&'static str];

    fn build_wgpu_pipeline_layouts(
        &self,
        wm: &WmRenderer
    ) -> HashMap<String, wgpu::PipelineLayout>;

    fn build_wgpu_pipelines(
        &self,
        wm: &WmRenderer,
    ) -> HashMap<String, wgpu::RenderPipeline>;

    fn render<'pipeline: 'render_pass, 'wm, 'pass_borrow, 'render_pass: 'pass_borrow, 'arena: 'pass_borrow + 'render_pass>(
        &'pipeline self,

        wm: &'wm WmRenderer,
        render_pass: &'pass_borrow mut wgpu::RenderPass<'render_pass>,
        arena: &'pass_borrow mut WmArena<'arena>
    );

}

pub struct RenderPipelineManager {
    pub pipeline_layouts: ArcSwap<HashMap<String, Arc<PipelineLayout>>>,
    pub render_pipelines: ArcSwap<HashMap<String, Arc<RenderPipeline>>>,

    pub shader_map: RwLock<HashMap<String, Box<dyn WmShader>>>,
    pub bind_group_layouts: RwLock<HashMap<String, BindGroupLayout>>,
    pub resource_provider: Arc<dyn ResourceProvider>
}

impl RenderPipelineManager {
    
    fn create_bind_group_layouts(device: &wgpu::Device) -> HashMap<String, BindGroupLayout> {
        [
            (
                "camera".into(),
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Camera Bind Group Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None
                                },
                                count: None
                            }
                        ]
                    }
                )
            ),
            (
                "texture".into(),
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Texture Bind Group Layout Descriptor"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false
                                },
                                count: None
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                                count: None
                            }
                        ]
                    }
                )
            ),
            (
                "cubemap".into(),
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Cubemap Bind Group Layout Descriptor"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension: wgpu::TextureViewDimension::Cube,
                                    multisampled: false
                                },
                                count: None
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                                count: None
                            }
                        ]
                    }
                )
            ),
            (
                "ssbo".into(),
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false,
                                    min_binding_size: None
                                },
                                count: None
                            }
                        ]
                    }
                )
            ),
            (
                "matrix4".into(),
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Mat4x4<f32> Bind Group Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None
                                },
                                count: None
                            }
                        ]
                    }
                )
            )
        ].into_iter().collect()
    }

    pub fn build_shaders(wm: &WmRenderer, wm_pipelines: &[&dyn WmPipeline]) -> HashMap<String, Box<dyn WmShader>> {
        wm_pipelines.iter().flat_map(|wm_pipeline| {
            wm_pipeline.provide_shaders(wm)
        }).collect()
    }

    pub fn build_pipeline_layouts(wm: &WmRenderer, wm_pipelines: &[&dyn WmPipeline]) -> HashMap<String, PipelineLayout> {
        wm_pipelines.iter().flat_map(|wm_pipeline| {
            wm_pipeline.build_wgpu_pipeline_layouts(wm)
        }).collect()
    }

    pub fn build_pipelines(wm: &WmRenderer, wm_pipelines: &[&dyn WmPipeline]) -> HashMap<String, RenderPipeline> {
        wm_pipelines.iter().flat_map(|wm_pipeline| {
            wm_pipeline.build_wgpu_pipelines(wm)
        }).collect()
    }

    pub fn new(resource_provider: Arc<dyn ResourceProvider>) -> Self {
        Self {
            pipeline_layouts: ArcSwap::new(Arc::new(HashMap::new())),
            render_pipelines: ArcSwap::new(Arc::new(HashMap::new())),
            resource_provider,
            bind_group_layouts: RwLock::new(HashMap::new()),
            shader_map: RwLock::new(HashMap::new())
        }

    }

    
    pub fn init(&self, wm: &WmRenderer, wm_pipelines: &[&dyn WmPipeline]) {
        {
            self.bind_group_layouts.write().extend(
                Self::create_bind_group_layouts(&wm.wgpu_state.device).into_iter()
            )
        }

        let pipeline_layouts = Self::build_pipeline_layouts(wm, wm_pipelines)
            .into_iter()
            .map(|(name, layout)| (name, Arc::new(layout)))
            .collect();

        self.pipeline_layouts.store(Arc::new(pipeline_layouts));

        let shaders = Self::build_shaders(wm, wm_pipelines);

        {
            *self.shader_map.write() = shaders;
        }

        let pipelines = Self::build_pipelines(wm, wm_pipelines)
            .into_iter()
            .map(|(name, layout)| (name, Arc::new(layout)))
            .collect();

        self.render_pipelines.store(Arc::new(pipelines));
    }

}
