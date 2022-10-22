use std::collections::HashMap;

use crate::mc::entity::{EntityInstanceVBOEntry, EntityInstances};
use crate::render::entity::EntityVertex;
use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

pub struct EntityPipeline<'entities> {
    pub entities: &'entities [&'entities EntityInstances],
}

impl<'frames> WmPipeline for EntityPipeline<'frames> {
    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/entity"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        HashMap::from([(
            "wgpu_mc:shaders/entity".into(),
            Box::new(
                WgslShader::init(
                    &"wgpu_mc:shaders/entity.wgsl".try_into().unwrap(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into(),
                )
                .unwrap(),
            ) as Box<dyn WmShader>,
        )])
    }

    fn atlases(&self) -> &'static [&'static str] {
        &["wgpu_mc:atlases/player_skins", "wgpu_mc:atlases/mobs"]
    }

    fn build_wgpu_pipeline_layouts(
        &self,
        wm: &WmRenderer,
    ) -> HashMap<String, wgpu::PipelineLayout> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.bind_group_layouts.read();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:layouts/entity".into(),
            wm.wgpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Entity Pipeline Layout"),
                    bind_group_layouts: &[
                        layouts.get("ssbo").unwrap(),
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
        let shader = shader_map.get("wgpu_mc:shaders/entity").unwrap();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:pipelines/entity".into(),
            wm.wgpu_state
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(layouts.get("wgpu_mc:layouts/entity").unwrap()),
                    vertex: wgpu::VertexState {
                        module: shader.get_vert().0,
                        entry_point: shader.get_vert().1,
                        buffers: &[EntityVertex::desc(), EntityInstanceVBOEntry::desc()],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Cw,
                        // cull_mode: Some(wgpu::Face::Back),
                        cull_mode: None,
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
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::OVER,
                                alpha: wgpu::BlendComponent::OVER,
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
        render_pass.set_pipeline(
            arena.alloc(
                wm.render_pipeline_manager
                    .load()
                    .render_pipelines
                    .load()
                    .get("wgpu_mc:pipelines/entity")
                    .unwrap()
                    .clone(),
            ),
        );

        self.entities.iter().for_each(|instances| {
            let uploaded = {
                let lock = instances.uploaded.read();
                arena.alloc(lock.as_ref().unwrap().clone())
            };

            let entity = arena.alloc(instances.entity.clone());

            //Bind the transform SSBO
            render_pass.set_bind_group(0, &uploaded.transform_ssbo.1, &[]);

            //Bind the entity texture atlas
            render_pass.set_bind_group(1, &entity.texture.bind_group, &[]);

            //Bind projection matrix
            render_pass.set_bind_group(
                2,
                (**arena.alloc(wm.mc.camera_bind_group.load_full()))
                    .as_ref()
                    .unwrap(),
                &[],
            );

            render_pass.set_vertex_buffer(0, entity.mesh.slice(..));

            render_pass.set_vertex_buffer(1, uploaded.instance_vbo.slice(..));

            render_pass.draw(
                0..instances.entity.vertices,
                0..instances.instances.len() as u32,
            );
        });
    }
}
