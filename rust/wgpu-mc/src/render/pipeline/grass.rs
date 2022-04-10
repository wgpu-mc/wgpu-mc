use std::collections::HashMap;
use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrassVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub lightmap_coords: [f32; 2],
    pub normal: [f32; 3],
    pub biome_color_coords: [f32; 2]
}

impl GrassVertex {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GrassVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                //Texcoords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                //Lightmap
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2
                },
            ],
        }
    }
}

pub struct GrassPipeline;

impl WmPipeline for GrassPipeline {

    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/grass"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [
            (
                "wgpu_mc:shaders/grass".into(),
                Box::new(WgslShader::init(
                    &"wgpu_mc:shaders/grass.wgsl".try_into().unwrap(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into()
                )) as Box<dyn WmShader>
            )
        ].into_iter().collect()
    }

    fn atlases(&self) -> &'static[&'static str] {
        &[
            "wgpu_mc:atlases/biome"
        ]
    }

    fn build_wgpu_pipeline_layouts(&self, wm: &WmRenderer) -> HashMap<String, wgpu::PipelineLayout> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.bind_group_layouts.read();

        let mut map = HashMap::new();

        map.insert("wgpu_mc:layouts/grass".into(), wm.wgpu_state.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Grass Pipeline Layout"),
                bind_group_layouts: &[
                    //Block atlas, projection matrix, biome atlas
                    layouts.get("texture").unwrap(), layouts.get("matrix4").unwrap(), layouts.get("texture").unwrap()
                ],
                push_constant_ranges: &[]
            }
        ));

        map
    }

    fn build_wgpu_pipelines(&self, wm: &WmRenderer) -> HashMap<String, RenderPipeline> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.pipeline_layouts.load_full();
        let shader_map = pipeline_manager.shader_map.read();
        let shader = shader_map.get("wgpu_mc:shaders/grass").unwrap();

        let mut map = HashMap::new();

        map.insert("wgpu_mc:pipelines/grass".into(), wm.wgpu_state.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(layouts.get("wgpu_mc:layouts/grass").unwrap()),
            vertex: wgpu::VertexState {
                module: shader.get_vert().0,
                entry_point: shader.get_vert().1,
                buffers: &[GrassVertex::desc()]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: Some(wgpu::FragmentState {
                module: shader.get_frag().0,
                entry_point: shader.get_frag().1,
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::OVER
                    }),
                    write_mask: Default::default()
                }]
            }),
            multiview: None
        }));

        map
    }

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, _wm: &'b WmRenderer, _render_pass: &'c mut RenderPass<'d>, _arena: &'c mut WmArena<'e>) {
        todo!()
    }

}