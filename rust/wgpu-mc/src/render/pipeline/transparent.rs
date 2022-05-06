use crate::render::pipeline::terrain::{TerrainVertex, BLOCK_ATLAS_NAME};
use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;
use std::collections::HashMap;

pub struct TransparentPipeline;

impl WmPipeline for TransparentPipeline {
    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/transparent"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [(
            "wgpu_mc:shaders/transparent".into(),
            Box::new(WgslShader::init(
                &"wgpu_mc:shaders/transparent.wgsl".try_into().unwrap(),
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
            "wgpu_mc:layouts/transparent".into(),
            wm.wgpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Transparent Pipeline Layout"),
                    bind_group_layouts: &[
                        //&layouts.texture, &layouts.matrix4, &layouts.get("cubemap").unwrap()
                        layouts.get("texture").unwrap(),
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
        let shader = shader_map.get("wgpu_mc:shaders/transparent").unwrap();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:pipelines/transparent".into(),
            wm.wgpu_state
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(layouts.get("wgpu_mc:layouts/transparent").unwrap()),
                    vertex: wgpu::VertexState {
                        module: shader.get_vert().0,
                        entry_point: shader.get_vert().1,
                        buffers: &[TerrainVertex::desc()],
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
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
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
        wm: &'b WmRenderer,
        render_pass: &'c mut RenderPass<'d>,
        arena: &'c mut WmArena<'e>,
    ) {
        let pipeline_manager = wm.render_pipeline_manager.load();
        let render_pipelines = pipeline_manager.render_pipelines.load();

        render_pass.set_pipeline(
            arena.alloc(
                render_pipelines
                    .get("wgpu_mc:pipelines/transparent")
                    .unwrap()
                    .clone(),
            ),
        );

        let block_atlas = arena.alloc(
            wm.mc
                .texture_manager
                .atlases
                .load()
                .get(BLOCK_ATLAS_NAME)
                .unwrap()
                .load(),
        );

        let bindable_texture = arena.alloc(block_atlas.bindable_texture.load_full());

        render_pass.set_bind_group(0, &bindable_texture.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            (**arena.alloc(wm.mc.camera_bind_group.load_full()))
                .as_ref()
                .unwrap(),
            &[],
        );

        let buffers = arena.alloc(wm.mc.chunks.section_buffers.load_full());
        let transparent = buffers.get("transparent").unwrap();

        [
            &transparent.north,
            &transparent.south,
            &transparent.top,
            &transparent.bottom,
            &transparent.west,
            &transparent.east,
            &transparent.other,
        ]
        .iter()
        .for_each(|&(buffer, verts)| {
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.draw(0..*verts as u32, 0..1);
        })
    }
}
