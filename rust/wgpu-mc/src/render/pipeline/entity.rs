use std::collections::HashMap;
use crate::render::entity::{EntityRenderInstance, EntityVertex};
use crate::render::entity::EntityGroupInstancingFrame;
use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

pub struct EntityPipeline<'frames> {
    pub frames: &'frames [&'frames EntityGroupInstancingFrame]
}

impl<'frames> WmPipeline for EntityPipeline<'frames> {

    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/entity"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [
            (
                "wgpu_mc:shaders/entity".into(),
                Box::new(WgslShader::init(
                    &"wgpu_mc:shaders/entity.wgsl".try_into().unwrap(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into()
                )) as Box<dyn WmShader>
            )
        ].into_iter().collect()
    }

    fn atlases(&self) -> &'static [&'static str] {
        &[
            "wgpu_mc:atlases/player_skins",
            "wgpu_mc:atlases/mobs"
        ]
    }

    fn build_wgpu_pipeline_layouts(&self, wm: &WmRenderer) -> HashMap<String, wgpu::PipelineLayout> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.bind_group_layouts.read();

        let mut map = HashMap::new();

        map.insert("wgpu_mc:layouts/entity".into(), wm.wgpu_state.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Entity Pipeline Layout"),
                bind_group_layouts: &[
                    layouts.get("ssbo").unwrap(),
                    layouts.get("ssbo").unwrap(),
                    layouts.get("texture").unwrap(),
                    layouts.get("matrix4").unwrap()
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
        let shader = shader_map.get("wgpu_mc:shaders/entity").unwrap();

        let mut map = HashMap::new();

        map.insert("wgpu_mc:pipelines/entity".into(), wm.wgpu_state.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(layouts.get("wgpu_mc:layouts/entity").unwrap()),
            vertex: wgpu::VertexState {
                module: shader.get_vert().0,
                entry_point: shader.get_vert().1,
                buffers: &[
                    EntityVertex::desc(),
                    EntityRenderInstance::desc()
                ]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
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
                        alpha: wgpu::BlendComponent::REPLACE
                    }),
                    write_mask: Default::default()
                }]
            }),
            multiview: None
        }));

        map
    }

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, wm: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
        render_pass.set_pipeline(
            arena.alloc(
                wm.render_pipeline_manager.load()
                    .render_pipelines.load()
                    .get("wgpu_mc:pipelines/entity")
                    .unwrap()
                    .clone()
            )
        );

        self.frames.iter().for_each(|instance_type| {
            render_pass.set_bind_group(
                0,
                arena.alloc(instance_type.part_transform_matrices.clone()),
                &[]
            );

            render_pass.set_bind_group(
                1,
                arena.alloc(instance_type.texture_offsets.clone()),
                &[]
            );

            render_pass.set_bind_group(
                2, &arena.alloc(instance_type.texture.clone()).bind_group,
                &[]
            );

            render_pass.set_bind_group(
                3,
                (**arena.alloc(wm.mc.camera_bind_group.load_full()))
                    .as_ref()
                    .unwrap(),
                &[]
            );

            render_pass.set_vertex_buffer(
                0,
                arena.alloc(instance_type.vbo.clone()).slice(..)
            );

            render_pass.set_vertex_buffer(
                1,
                arena.alloc(instance_type.instance_vbo.clone()).slice(..)
            );

            render_pass.draw(0..instance_type.vertex_count, 0..instance_type.instance_count);
        });
    }

}