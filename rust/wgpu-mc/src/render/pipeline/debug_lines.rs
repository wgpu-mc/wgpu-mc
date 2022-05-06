use crate::render::pipeline::WmPipeline;
use crate::render::shader::{WgslShader, WmShader};
use std::collections::HashMap;
use wgpu::util::BufferInitDescriptor;

use crate::camera::UniformMatrixHelper;
use crate::util::WmArena;
use crate::wgpu::util::DeviceExt;
use crate::wgpu::{RenderPass, RenderPipeline, RenderPipelineDescriptor};
use crate::WmRenderer;
use bytemuck::{Pod, Zeroable};
use cgmath::Rad;
use wgpu::{BindGroupDescriptor, BindGroupEntry};

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
/// Data to describe an instance of an entity type on the GPU
pub struct DebugLineVertex {
    /// Index into mat4[][]
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl DebugLineVertex {
    const VAA: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DebugLineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VAA,
        }
    }
}

pub struct DebugLinesPipeline;

impl WmPipeline for DebugLinesPipeline {
    fn name(&self) -> &'static str {
        "wgpu_mc:pipelines/debug_lines"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [(
            "wgpu_mc:shaders/debug_lines".into(),
            Box::new(WgslShader::init(
                &"wgpu_mc:shaders/debug_lines.wgsl".try_into().unwrap(),
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
            "wgpu_mc:layouts/debug_lines".into(),
            wm.wgpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Debug Lines Pipeline Layout"),
                    bind_group_layouts: &[layouts.get("matrix4").unwrap()],
                    push_constant_ranges: &[],
                }),
        );

        map
    }

    fn build_wgpu_pipelines(&self, wm: &WmRenderer) -> HashMap<String, RenderPipeline> {
        let pipeline_manager = wm.render_pipeline_manager.load_full();
        let layouts = &pipeline_manager.pipeline_layouts.load_full();
        let shader_map = pipeline_manager.shader_map.read();
        let shader = shader_map.get("wgpu_mc:shaders/debug_lines").unwrap();

        let mut map = HashMap::new();

        map.insert(
            "wgpu_mc:pipelines/debug_lines".into(),
            wm.wgpu_state
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(layouts.get("wgpu_mc:layouts/debug_lines").unwrap()),
                    vertex: wgpu::VertexState {
                        module: shader.get_vert().0,
                        entry_point: shader.get_vert().1,
                        buffers: &[DebugLineVertex::desc()],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::LineList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: Default::default(),
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
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
        let render_pipelines = pipeline_manager.render_pipelines.load();

        render_pass.set_pipeline(
            arena.alloc(
                render_pipelines
                    .get("wgpu_mc:pipelines/debug_lines")
                    .unwrap()
                    .clone(),
            ),
        );

        let vertices: &[f32] = &[
            //+Y is up in mc
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.2, 0.0, 0.0, 1.0, 0.0, //-X is west in mc
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -0.2, 0.0, 0.0, 1.0, 0.0, 0.0,
            //-Z is north in mc
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -0.2, 0.0, 0.0, 1.0,
        ];

        let camera = *wm.mc.camera.load_full();

        let rotation_matrix = cgmath::Matrix4::<f32>::from_angle_x(Rad(-camera.pitch))
            * cgmath::Matrix4::<f32>::from_angle_y(Rad(camera.yaw));

        let helper = UniformMatrixHelper {
            view_proj: rotation_matrix.into(),
        };

        let rotation_buffer = arena.alloc(wm.wgpu_state.device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&helper),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        ));

        let rotation_matrix_bind_group = arena.alloc(
            wm.wgpu_state
                .device
                .create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: wm
                        .render_pipeline_manager
                        .load_full()
                        .bind_group_layouts
                        .read()
                        .get("matrix4")
                        .unwrap(),
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: rotation_buffer.as_entire_binding(),
                    }],
                }),
        );

        let vertex_buffer = arena.alloc(wm.wgpu_state.device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        render_pass.set_bind_group(0, rotation_matrix_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
