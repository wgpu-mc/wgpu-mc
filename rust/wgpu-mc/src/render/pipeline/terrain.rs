use std::collections::HashMap;

use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use crate::util::WmArena;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;

pub struct TerrainPipeline;

pub const BLOCK_ATLAS_NAME: &str = "wgpu_mc:atlases/block";

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub lightmap_coords: [f32; 2],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub tangent: [f32; 4],
    pub uv_offset: u32,
}

impl TerrainVertex {
    const VAA: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Uint32
    ];

    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VAA,
        }
    }
}

impl WmPipeline for TerrainPipeline {
    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/terrain"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [(
            "wgpu_mc:shaders/terrain".into(),
            Box::new(
                WgslShader::init(
                    &"wgpu_mc:shaders/terrain.wgsl".try_into().unwrap(),
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
        &["wgpu_mc:atlases/block"]
    }

    fn build_wgpu_pipeline_layouts(
        &self,
        wm: &WmRenderer,
    ) -> HashMap<String, wgpu::PipelineLayout> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.bind_group_layouts.read();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:layouts/terrain".into(),
            wm.wgpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Terrain Pipeline Layout"),
                    bind_group_layouts: &[
                        //&layouts.texture, &layouts.matrix4, &layouts.cubemap
                        layouts.get("texture").unwrap(),
                        layouts.get("matrix4").unwrap(),
                        // layouts.get("ssbo").unwrap(),
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
        let shader = shader_map.get("wgpu_mc:shaders/terrain").unwrap();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:pipelines/terrain".into(),
            wm.wgpu_state
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(layouts.get("wgpu_mc:layouts/terrain").unwrap()),
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
        let pipeline_manager = wm.render_pipeline_manager.load();
        let render_pipelines = pipeline_manager.render_pipelines.load();

        render_pass.set_pipeline(
            arena.alloc(
                render_pipelines
                    .get("wgpu_mc:pipelines/terrain")
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

        //TODO
        // render_pass.set_bind_group(
        //     2,
        //     (**arena.alloc(wm.mc.animated_block_bind_group.load_full()))
        //         .as_ref()
        //         .unwrap(),
        //     &[],
        // );

        let buffers = arena.alloc(wm.mc.chunks.section_buffers.load_full());
        let terrain = buffers.get("terrain").unwrap();

        [
            &terrain.north,
            &terrain.south,
            &terrain.top,
            &terrain.bottom,
            &terrain.west,
            &terrain.east,
            &terrain.other,
        ]
        .iter()
        .for_each(|&(buffer, verts)| {
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.draw(0..*verts as u32, 0..1);
        })
    }
}
