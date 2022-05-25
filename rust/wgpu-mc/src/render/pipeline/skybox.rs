use std::collections::HashMap;
use wgpu::DepthStencilState;
use wgpu::util::DeviceExt;

use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::render::world::sky::{SkyboxVertex, SkyVertex};
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

const VERTICES: [SkyboxVertex; 36] = [
    SkyboxVertex {
        position: e,
        //south 2nd uv
        uv: [self.textures.south.1 .0, self.textures.south.1 .1]
    },
    SkyboxVertex {
        position: h,
        uv: [self.textures.south.1 .0, self.textures.south.0 .1],
    },
    SkyboxVertex {
        position: f,
        uv: [self.textures.south.0 .0, self.textures.south.1 .1],
    },
    SkyboxVertex {
        position: h,
        uv: [self.textures.south.1 .0, self.textures.south.0 .1],
    },
    SkyboxVertex {
        position: g,
        uv: [self.textures.south.0 .0, self.textures.south.0 .1],
    },
    SkyboxVertex {
        position: f,
        uv: [self.textures.south.0 .0, self.textures.south.1 .1],
    },
    SkyboxVertex {
        position: g,
        uv: [self.textures.west.1 .0, self.textures.west.0 .1],
    },
    SkyboxVertex {
        position: b,
        uv: [self.textures.west.0 .0, self.textures.west.1 .1],
    },
    SkyboxVertex {
        position: f,
        uv: [self.textures.west.1 .0, self.textures.west.1 .1],
    },
    SkyboxVertex {
        position: c,
        uv: [self.textures.west.0 .0, self.textures.west.0 .1],
    },
    SkyboxVertex {
        position: b,
        uv: [self.textures.west.0 .0, self.textures.west.1 .1],
    },
    SkyboxVertex {
        position: g,
        uv: [self.textures.west.1 .0, self.textures.west.0 .1],
    },
    SkyboxVertex {
        position: c,
        uv: [self.textures.north.1 .0, self.textures.north.0 .1],
    },
    SkyboxVertex {
        position: a,
        uv: [self.textures.north.0 .0, self.textures.north.1 .1],
    },
    SkyboxVertex {
        position: b,
        uv: [self.textures.north.1 .0, self.textures.north.1 .1],
    },
    SkyboxVertex {
        position: d,
        uv: [self.textures.north.0 .0, self.textures.north.0 .1],
    },
    SkyboxVertex {
        position: a,
        uv: [self.textures.north.0 .0, self.textures.north.1 .1],
    },
    SkyboxVertex {
        position: c,
        uv: [self.textures.north.1 .0, self.textures.north.0 .1],
    },
    SkyboxVertex {
        position: e,
        uv: [self.textures.east.0 .0, self.textures.east.1 .1],
    },
    SkyboxVertex {
        position: a,
        uv: [self.textures.east.1 .0, self.textures.east.1 .1],
    },
    SkyboxVertex {
        position: d,
        uv: [self.textures.east.1 .0, self.textures.east.0 .1],
    },
    SkyboxVertex {
        position: d,
        uv: [self.textures.east.1 .0, self.textures.east.0 .1],
    },
    SkyboxVertex {
        position: h,
        uv: [self.textures.east.0 .0, self.textures.east.0 .1],
    },
    SkyboxVertex {
        position: e,
        uv: [self.textures.east.0 .0, self.textures.east.1 .1],
    },
    SkyboxVertex {
        position: g,
        uv: [self.textures.up.1 .0, self.textures.up.0 .1],
    },
    SkyboxVertex {
        position: h,
        uv: [self.textures.up.0 .0, self.textures.up.0 .1],
    },
    SkyboxVertex {
        position: d,
        uv: [self.textures.up.0 .0, self.textures.up.1 .1],
    },
    SkyboxVertex {
        position: c,
        uv: [self.textures.up.1 .0, self.textures.up.1 .1],
    },
    SkyboxVertex {
        position: g,
        uv: [self.textures.up.1 .0, self.textures.up.0 .1],
    },
    SkyboxVertex {
        position: d,
        uv: [self.textures.up.0 .0, self.textures.up.1 .1],
    },
    SkyboxVertex {
        position: a,
        uv: [self.textures.down.1 .0, self.textures.down.0 .1],
    },
    SkyboxVertex {
        position: b,
        uv: [self.textures.down.0 .0, self.textures.down.0 .1],
        normal: [0.0, -1.0, 0.0],
        part_id,
    },
    SkyboxVertex {
        position: f,
        uv: [self.textures.down.0 .0, self.textures.down.1 .1],
        normal: [0.0, -1.0, 0.0],
        part_id,
    },
    SkyboxVertex {
        position: e,
        uv: [self.textures.down.1 .0, self.textures.down.1 .1],
        normal: [0.0, -1.0, 0.0],
        part_id,
    },
    SkyboxVertex {
        position: a,
        uv: [self.textures.down.1 .0, self.textures.down.0 .1],
        normal: [0.0, -1.0, 0.0],
        part_id,
    },
    SkyboxVertex {
        position: f,
        uv: [self.textures.down.0 .0, self.textures.down.1 .1],
        normal: [0.0, -1.0, 0.0],
        part_id,
    }
];

pub struct SkyboxPipeline;

impl WmPipeline for SkyboxPipeline {
    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/skybox"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [(
            "wgpu_mc:shaders/skybox".into(),
            Box::new(WgslShader::init(
                &"wgpu_mc:shaders/skybox.wgsl".try_into().unwrap(),
                &*wm.mc.resource_provider,
                &wm.wgpu_state.device,
                "fs_main".into(),
                "vs_main".into(),
            ).unwrap()) as Box<dyn WmShader>,
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
            "wgpu_mc:layouts/skybox".into(),
            wm.wgpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Skybox Pipeline Layout"),
                    bind_group_layouts: &[
                        layouts.get("matrix4").unwrap(),
                        layouts.get("texture").unwrap()
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
        let shader = shader_map.get("wgpu_mc:shaders/skybox").unwrap();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:pipelines/skybox".into(),
            wm.wgpu_state
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(layouts.get("wgpu_mc:layouts/skybox").unwrap()),
                    vertex: wgpu::VertexState {
                        module: shader.get_vert().0,
                        entry_point: shader.get_vert().1,
                        buffers: &[SkyboxVertex::desc()],
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

        let vertices = 

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
