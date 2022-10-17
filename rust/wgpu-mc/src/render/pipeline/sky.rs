use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::render::world::sky::SkyVertex;
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

pub struct SkyPipeline;

impl WmPipeline for SkyPipeline {
    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/sky"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [(
            "wgpu_mc:shaders/sky".into(),
            Box::new(
                WgslShader::init(
                    &"wgpu_mc:shaders/sky.wgsl".try_into().unwrap(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into(),
                )
                .unwrap(),
            ) as Box<dyn WmShader>,
        )]
        .into_iter()
        .collect()
    }

    fn atlases(&self) -> &'static [&'static str] {
        &[]
    }

    fn build_wgpu_pipeline_layouts(
        &self,
        wm: &WmRenderer,
    ) -> HashMap<String, wgpu::PipelineLayout> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.bind_group_layouts.read();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:layouts/sky".into(),
            wm.wgpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Sky Pipeline Layout"),
                    bind_group_layouts: &[
                        // layouts.get("cubemap").unwrap(),
                        layouts.get("matrix4").unwrap(),
                    ],
                    push_constant_ranges: &[],
                }),
        );

        map
    }

    fn build_wgpu_pipelines(&self, wm: &WmRenderer) -> HashMap<String, RenderPipeline> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.pipeline_layouts.load_full();
        let shader_map = pipeline_manager.shader_map.read();
        let shader = shader_map.get("wgpu_mc:shaders/sky").unwrap();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:pipelines/sky".into(),
            wm.wgpu_state
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(layouts.get("wgpu_mc:layouts/sky").unwrap()),
                    vertex: wgpu::VertexState {
                        module: shader.get_vert().0,
                        entry_point: shader.get_vert().1,
                        buffers: &[SkyVertex::desc()],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: Default::default(),
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Always,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: shader.get_frag().0,
                        entry_point: shader.get_frag().1,
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: Default::default(),
                        })],
                    }),
                    multiview: None,
                }),
        );

        map
    }

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(
        &'a self,
        wm: &'b WmRenderer,
        render_pass: &'c mut RenderPass<'d>,
        arena: &'c mut WmArena<'e>,
    ) {
        let pipeline_manager = wm.render_pipeline_manager.load();

        let vertices = [
            SkyVertex {
                position: [1.0, 1.0, 0.5],
            },
            SkyVertex {
                position: [-1.0, 1.0, 0.5],
            },
            SkyVertex {
                position: [-1.0, -1.0, 0.5],
            },
            SkyVertex {
                position: [-1.0, -1.0, 0.5],
            },
            SkyVertex {
                position: [1.0, 1.0, 0.5],
            },
            SkyVertex {
                position: [-1.0, 1.0, 0.5],
            },
        ];

        let vertex_buffer = arena.alloc(wm.wgpu_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        render_pass.set_pipeline(
            arena.alloc(
                pipeline_manager
                    .render_pipelines
                    .load()
                    .get("wgpu_mc:pipelines/sky")
                    .unwrap()
                    .clone(),
            ),
        );

        let opt = arena.alloc(wm.mc.camera_bind_group.load_full());
        let bg = (**opt).as_ref().unwrap();

        render_pass.set_bind_group(0, bg, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
