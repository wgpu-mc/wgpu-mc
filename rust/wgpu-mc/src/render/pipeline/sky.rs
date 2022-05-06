use std::collections::HashMap;

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
            Box::new(WgslShader::init(
                &"wgpu_mc:shaders/sky.wgsl".try_into().unwrap(),
                &*wm.mc.resource_provider,
                &wm.wgpu_state.device,
                "fs_main".into(),
                "vs_main".into(),
            )) as Box<dyn WmShader>,
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
                        layouts.get("cubemap").unwrap(),
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
                    layout: Some(layouts.get("sky").unwrap()),
                    vertex: wgpu::VertexState {
                        module: shader.get_vert().0,
                        entry_point: shader.get_vert().1,
                        buffers: &[SkyVertex::desc()],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: Default::default(),
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: shader.get_frag().0,
                        entry_point: shader.get_frag().1,
                        targets: &[wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: Default::default(),
                        }],
                    }),
                    multiview: None,
                }),
        );

        map
    }

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(
        &'a self,
        _wm: &'b WmRenderer,
        _render_pass: &'c mut RenderPass<'d>,
        _arena: &'c mut WmArena<'e>,
    ) {
        todo!()
    }
}
