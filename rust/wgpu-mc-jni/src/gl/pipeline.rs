use std::collections::HashMap;

use std::sync::Arc;

use arc_swap::ArcSwap;
use cgmath::Matrix4;
use futures::StreamExt;
use once_cell::sync::OnceCell;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroupDescriptor, BindGroupEntry, RenderPass, RenderPipeline, VertexState};
use wgpu_biolerless::{
    FragmentShaderState, ModuleSrc, PipelineBuilder, ShaderModuleSources, VertexShaderState,
};

use wgpu_mc::camera::UniformMatrixHelper;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::render::shader::{WgslShader, WmShader};
use wgpu_mc::texture::{BindableTexture, TextureSamplerView};
use wgpu_mc::util::WmArena;
use wgpu_mc::wgpu::PipelineLayout;
use wgpu_mc::{wgpu, WmRenderer};

use crate::wgpu::{
    BlendComponent, BlendState, BufferUsages, ColorTargetState, DepthStencilState, PrimitiveState,
};
use crate::{gl, Extent3d};

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Clone, Debug)]
pub enum GLCommand {
    SetMatrix(Matrix4<f32>),
    ClearColor(f32, f32, f32),
    UsePipeline(usize),
    SetVertexBuffer(Vec<u8>),
    SetIndexBuffer(Vec<u32>),
    DrawIndexed(u32),
    Draw(u32),
    AttachTexture(i32, i32),
}

#[derive(Debug)]
pub struct TextureUnit {
    pub target_tex_2d: i32,
    // target_tex_3d: i32
}

#[derive(Debug)]
pub struct GlPipeline {
    pub commands: ArcSwap<Vec<GLCommand>>,
    pub blank_texture: OnceCell<Arc<BindableTexture>>,
}

fn byte_buffer_to_short(bytes: &[u8]) -> Vec<u16> {
    bytes.iter().map(|byte| *byte as u16).collect()
}

impl WmPipeline for GlPipeline {
    fn name(&self) -> &'static str {
        "OpenGL"
    }

    fn provide_shaders(&self, wm: &WmRenderer) -> HashMap<String, Box<dyn WmShader>> {
        [
            (
                "wgpu_mc_ogl:shaders/pos_col_float3".into(),
                Box::new(
                    WgslShader::init(
                        &("wgpu_mc", "shaders/gui_col_pos.wgsl").into(),
                        &*wm.mc.resource_provider,
                        wm.wgpu_state.device(),
                        "fs_main".into(),
                        "vs_main".into(),
                    )
                    .unwrap(),
                ) as Box<dyn WmShader>,
            ),
            (
                "wgpu_mc_ogl:shaders/pos_col_uint".into(),
                Box::new(
                    WgslShader::init(
                        &("wgpu_mc", "shaders/gui_col_pos_uint.wgsl").into(),
                        &*wm.mc.resource_provider,
                        wm.wgpu_state.device(),
                        "fs_main".into(),
                        "vs_main".into(),
                    )
                    .unwrap(),
                ) as Box<dyn WmShader>,
            ),
            (
                "wgpu_mc_ogl:shaders/pos_tex".into(),
                Box::new(
                    WgslShader::init(
                        &("wgpu_mc", "shaders/gui_uv_pos.wgsl").into(),
                        &*wm.mc.resource_provider,
                        wm.wgpu_state.device(),
                        "fs_main".into(),
                        "vs_main".into(),
                    )
                    .unwrap(),
                ) as Box<dyn WmShader>,
            ),
            (
                "wgpu_mc_ogl:shaders/clearcolor".into(),
                Box::new(
                    WgslShader::init(
                        &("wgpu_mc", "shaders/clearcolor.wgsl").into(),
                        &*wm.mc.resource_provider,
                        wm.wgpu_state.device(),
                        "fs_main".into(),
                        "vs_main".into(),
                    )
                    .unwrap(),
                ) as Box<dyn WmShader>,
            ),
            (
                "wgpu_mc_ogl:shaders/pos_color_uv_light".into(),
                Box::new(
                    WgslShader::init(
                        &("wgpu_mc", "shaders/gui_pos_color_uv_light.wgsl").into(),
                        &*wm.mc.resource_provider,
                        wm.wgpu_state.device(),
                        "fs_main".into(),
                        "vs_main".into(),
                    )
                    .unwrap(),
                ) as Box<dyn WmShader>,
            ),
            (
                "wgpu_mc_ogl:shaders/pos_texture_color".into(),
                Box::new(
                    WgslShader::init(
                        &("wgpu_mc", "shaders/gui_pos_texture_color.wgsl").into(),
                        &*wm.mc.resource_provider,
                        wm.wgpu_state.device(),
                        "fs_main".into(),
                        "vs_main".into(),
                    )
                    .unwrap(),
                ) as Box<dyn WmShader>,
            ),
        ]
        .into_iter()
        .collect()
    }

    fn atlases(&self) -> &'static [&'static str] {
        &[]
    }

    fn build_wgpu_pipeline_layouts(&self, wm: &WmRenderer) -> HashMap<String, PipelineLayout> {
        let pipeline_manager = wm.render_pipeline_manager.load();
        let layouts = pipeline_manager.bind_group_layouts.read();

        [
            (
                "wgpu_mc_ogl:layouts/pos_col".into(),
                wm.wgpu_state.create_pipeline_layout(
                    Some("pos_col"),
                    &[layouts.get("matrix4").unwrap()],
                    &[],
                ),
            ),
            (
                "wgpu_mc_ogl:layouts/pos_tex".into(),
                wm.wgpu_state.create_pipeline_layout(
                    Some("pos_tex"),
                    &[
                        layouts.get("matrix4").unwrap(),
                        layouts.get("texture").unwrap(),
                    ],
                    &[],
                ),
            ),
            (
                "wgpu_mc_ogl:layouts/clearcolor".into(),
                wm.wgpu_state
                    .create_pipeline_layout(Some("clearcolor"), &[], &[]),
            ),
        ]
        .into_iter()
        .collect()
    }

    fn build_wgpu_pipelines(&self, wm: &WmRenderer) -> HashMap<String, RenderPipeline> {
        let pipeline_manager = wm.render_pipeline_manager.load();
        let layouts = pipeline_manager.pipeline_layouts.load();
        let shaders = pipeline_manager.shader_map.read();

        //Initialize the blank texture if necessary
        self.blank_texture.get_or_init(|| {
            let blank_tsv = TextureSamplerView::from_rgb_bytes(
                &wm.wgpu_state,
                &[0u8; 4],
                (1, 1),
                wgpu::TextureFormat::Bgra8Unorm,
            )
            .unwrap();

            Arc::new(BindableTexture::from_tsv(
                &wm.wgpu_state,
                &*pipeline_manager,
                blank_tsv,
            ))
        });

        let pos_col_float3_shader = shaders.get("wgpu_mc_ogl:shaders/pos_col_float3").unwrap();
        let pos_col_uint_shader = shaders.get("wgpu_mc_ogl:shaders/pos_col_uint").unwrap();
        let pos_color_uv_light_shader = shaders
            .get("wgpu_mc_ogl:shaders/pos_color_uv_light")
            .unwrap();
        let pos_texture_color_shader = shaders
            .get("wgpu_mc_ogl:shaders/pos_texture_color")
            .unwrap();
        let pos_tex_shader = shaders.get("wgpu_mc_ogl:shaders/pos_tex").unwrap();
        let clearcolor_shader = shaders.get("wgpu_mc_ogl:shaders/clearcolor").unwrap();

        [
            (
                "wgpu_mc_ogl:pipelines/pos_col_float3".into(),
                wm.wgpu_state.create_pipeline(
                    PipelineBuilder::new()
                        .layout(layouts.get("wgpu_mc_ogl:layouts/pos_col").unwrap())
                        .vertex(VertexShaderState {
                            entry_point: pos_col_float3_shader.get_vert().1,
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: 24,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 12,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                        })
                        .primitive(PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: true,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        })
                        .depth_stencil(DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: Default::default(),
                            bias: Default::default(),
                        })
                        .fragment(FragmentShaderState {
                            entry_point: pos_col_float3_shader.get_frag().1,
                            targets: &[Some(ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: Default::default(),
                            })],
                        })
                        .shader_src(ShaderModuleSources::Multi(
                            ModuleSrc::Ref(pos_col_float3_shader.get_vert().0),
                            ModuleSrc::Ref(pos_col_float3_shader.get_frag().0),
                        )),
                ),
            ),
            (
                "pos_tex".into(),
                wm.wgpu_state.create_pipeline(
                    PipelineBuilder::new()
                        .layout(layouts.get("wgpu_mc_ogl:layouts/pos_tex").unwrap())
                        .vertex(VertexShaderState {
                            // module: pos_tex_shader.get_vert().0,
                            entry_point: pos_tex_shader.get_vert().1,
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: 20,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x2,
                                        offset: 12,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                        })
                        .primitive(PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: true,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        })
                        .depth_stencil(DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: Default::default(),
                            bias: Default::default(),
                        })
                        .fragment(FragmentShaderState {
                            // module: pos_tex_shader.get_frag().0,
                            entry_point: pos_tex_shader.get_frag().1,
                            targets: &[Some(ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: Default::default(),
                            })],
                        })
                        .shader_src(ShaderModuleSources::Multi(
                            ModuleSrc::Ref(pos_tex_shader.get_vert().0),
                            ModuleSrc::Ref(pos_tex_shader.get_frag().0),
                        )),
                ),
            ),
            (
                "pos_col_uint".into(),
                wm.wgpu_state.create_pipeline(
                    PipelineBuilder::new()
                        .layout(layouts.get("wgpu_mc_ogl:layouts/pos_col").unwrap())
                        .vertex(VertexShaderState {
                            entry_point: pos_col_uint_shader.get_vert().1,
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: 16,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Uint32,
                                        offset: 12,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                        })
                        .primitive(PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: true,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        })
                        .depth_stencil(DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: Default::default(),
                            bias: Default::default(),
                        })
                        .fragment(FragmentShaderState {
                            entry_point: pos_col_uint_shader.get_frag().1,
                            targets: &[Some(ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: Default::default(),
                            })],
                        })
                        .shader_src(ShaderModuleSources::Multi(
                            ModuleSrc::Ref(pos_col_uint_shader.get_vert().0),
                            ModuleSrc::Ref(pos_col_uint_shader.get_frag().0),
                        )),
                ),
            ),
            (
                "clearcolor".into(),
                wm.wgpu_state.create_pipeline(
                    PipelineBuilder::new()
                        .layout(layouts.get("wgpu_mc_ogl:layouts/clearcolor").unwrap())
                        .vertex(VertexShaderState {
                            entry_point: clearcolor_shader.get_vert().1,
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: 20,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x2,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 8,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                        })
                        .primitive(PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: true,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        })
                        .depth_stencil(DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: Default::default(),
                            bias: Default::default(),
                        })
                        .fragment(FragmentShaderState {
                            entry_point: clearcolor_shader.get_frag().1,
                            targets: &[Some(ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: None,
                                write_mask: Default::default(),
                            })],
                        })
                        .shader_src(ShaderModuleSources::Multi(
                            ModuleSrc::Ref(clearcolor_shader.get_vert().0),
                            ModuleSrc::Ref(clearcolor_shader.get_frag().0),
                        )),
                ),
            ),
            (
                "wgpu_mc_ogl:pipelines/pos_color_uv_light".into(),
                wm.wgpu_state.create_pipeline(
                    PipelineBuilder::new()
                        .layout(layouts.get("wgpu_mc_ogl:layouts/pos_tex").unwrap())
                        .vertex(VertexShaderState {
                            entry_point: pos_color_uv_light_shader.get_vert().1,
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: 28,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Uint32,
                                        offset: 12,
                                        shader_location: 1,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x2,
                                        offset: 16,
                                        shader_location: 2,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Uint32,
                                        offset: 24,
                                        shader_location: 3,
                                    },
                                ],
                            }],
                        })
                        .primitive(PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: true,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        })
                        .depth_stencil(DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: Default::default(),
                            bias: Default::default(),
                        })
                        .fragment(FragmentShaderState {
                            entry_point: pos_color_uv_light_shader.get_frag().1,
                            targets: &[Some(ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: Default::default(),
                            })],
                        })
                        .shader_src(ShaderModuleSources::Multi(
                            ModuleSrc::Ref(pos_color_uv_light_shader.get_vert().0),
                            ModuleSrc::Ref(pos_color_uv_light_shader.get_frag().0),
                        )),
                ),
            ),
            (
                "wgpu_mc_ogl:pipelines/pos_texture_color".into(),
                wm.wgpu_state.create_pipeline(
                    PipelineBuilder::new()
                        .layout(layouts.get("wgpu_mc_ogl:layouts/pos_tex").unwrap())
                        .vertex(VertexShaderState {
                            entry_point: pos_texture_color_shader.get_vert().1,
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: 24,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x2,
                                        offset: 12,
                                        shader_location: 1,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Uint32,
                                        offset: 20,
                                        shader_location: 2,
                                    },
                                ],
                            }],
                        })
                        .primitive(PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: true,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        })
                        .depth_stencil(DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: Default::default(),
                            bias: Default::default(),
                        })
                        .fragment(FragmentShaderState {
                            // module: ,
                            entry_point: pos_color_uv_light_shader.get_frag().1,
                            targets: &[Some(ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: Default::default(),
                            })],
                        })
                        .shader_src(ShaderModuleSources::Multi(
                            ModuleSrc::Ref(pos_texture_color_shader.get_vert().0),
                            ModuleSrc::Ref(pos_color_uv_light_shader.get_frag().0),
                        )),
                ),
            ),
        ]
        .into()
    }

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'd>(
        &'a self,
        wm: &'b WmRenderer,
        render_pass: &'c mut RenderPass<'d>,
        arena: &'c mut WmArena<'e>,
    ) {
        let pipeline_manager = wm.render_pipeline_manager.load();
        let gl_alloc = gl::GL_ALLOC.get().unwrap().read();

        let commands = self.commands.load();

        commands.iter().for_each(|command| {
            match command {
                GLCommand::UsePipeline(pipeline) => render_pass.set_pipeline(
                    arena.alloc(
                        pipeline_manager
                            .render_pipelines
                            .load()
                            .get(match pipeline {
                                0 => "pos_col_uint",
                                1 => "pos_tex",
                                2 => "wgpu_mc_ogl:pipelines/pos_col_float3",
                                3 => "wgpu_mc_ogl:pipelines/pos_color_uv_light",
                                4 => "wgpu_mc_ogl:pipelines/pos_texture_color",
                                _ => unimplemented!(),
                            })
                            .unwrap()
                            .clone(),
                    ),
                ),
                GLCommand::SetVertexBuffer(buf) => {
                    let buffer = wm.wgpu_state.create_buffer(None, buf, BufferUsages::VERTEX);

                    render_pass.set_vertex_buffer(0, arena.alloc(buffer).slice(..));
                }
                GLCommand::SetIndexBuffer(buf) => {
                    let buffer = wm.wgpu_state.create_buffer(None, buf, BufferUsages::INDEX);

                    render_pass
                        .set_index_buffer(arena.alloc(buffer).slice(..), wgpu::IndexFormat::Uint32);
                }
                GLCommand::Draw(count) => {
                    render_pass.draw(0..*count, 0..1);
                }
                GLCommand::DrawIndexed(count) => {
                    render_pass.draw_indexed(0..*count, 0, 0..1);
                }
                GLCommand::ClearColor(r, g, b) => {
                    let (r, g, b) = (*r, *g, *b);

                    let vertex_buffer = arena.alloc(wm.wgpu_state.create_buffer(
                        None,
                        &[
                            -1.0, -1.0, r, g, b, -1.0, 1.0, r, g, b, 1.0, 1.0, r, g, b, -1.0, -1.0,
                            r, g, b, 1.0, 1.0, r, g, b, 1.0, -1.0, r, g, b,
                        ],
                        BufferUsages::VERTEX,
                    ));

                    render_pass.set_pipeline(
                        arena.alloc(
                            pipeline_manager
                                .render_pipelines
                                .load()
                                .get("clearcolor")
                                .unwrap()
                                .clone(),
                        ),
                    );

                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.draw(0..6, 0..1);
                }
                GLCommand::AttachTexture(slot, texture) => {
                    let texture = match gl_alloc.get(texture) {
                        None => self.blank_texture.get().unwrap().clone(),
                        Some(tx) => tx.bindable_texture.as_ref().unwrap().clone(),
                    };

                    render_pass.set_bind_group(
                        (slot + 1) as u32,
                        &arena.alloc(texture).bind_group,
                        &[],
                    );
                }
                GLCommand::SetMatrix(mat) => {
                    let buffer = arena.alloc(wm.wgpu_state.create_buffer(
                        None,
                        &[&UniformMatrixHelper {
                            view_proj: (*mat).into(),
                        }],
                        BufferUsages::UNIFORM,
                    ));

                    let bg = arena.alloc(
                        wm.wgpu_state.create_bind_group(
                            None,
                            pipeline_manager
                                .bind_group_layouts
                                .read()
                                .get("matrix4")
                                .unwrap(),
                            &[BindGroupEntry {
                                binding: 0,
                                resource: buffer.as_entire_binding(),
                            }],
                        ),
                    );

                    render_pass.set_bind_group(0, bg, &[]);
                }
                _ => {}
            };
        });
    }
}
