use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;

use arc_swap::ArcSwap;
use cgmath::{Matrix4, SquareMatrix};
use futures::StreamExt;
use wgpu::{BindGroupDescriptor, BindGroupEntry, PipelineLayoutDescriptor, RenderPass, RenderPipeline, VertexState};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use wgpu_mc::{wgpu, WmRenderer};
use wgpu_mc::camera::UniformMatrixHelper;
use wgpu_mc::model::BindableTexture;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::render::shader::{GlslShader, WgslShader, WmShader};
use wgpu_mc::texture::TextureSamplerView;
use wgpu_mc::util::WmArena;
use wgpu_mc::wgpu::PipelineLayout;

use crate::gl::{get_buffer, GL_ALLOC, GlAttributeFormat, GlAttributeType, GlResource};
use crate::gl;

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Clone, Debug)]
pub enum GLCommand {
    BindTexture(i32),
    BindBuffer(i32, i32),
    ActiveTexture(i32),
    DrawArray(i32, i32, i32),
    PushMatrix,
    PopMatrix,
    VertexPointer(i32, i32, i32, *const u8),
    ColorPointer(i32, i32, i32, *const u8),
    TexCoordPointer(i32, i32, i32, *const u8),
    BindVertexArray(i32),
    TexImage2D((RefCell<Option<Vec<u8>>>, u32, u32, wgpu::TextureFormat)),
    EnableClientState(u32),
    DisableClientState(u32),
    MultMatrix(Matrix4<f32>),
    SetMatrix(Matrix4<f32>),
    MatrixMode(usize),
    DrawElements(i32, i32, i32, *const u8),
    ClearColor(f32, f32, f32),
    BufferData(RefCell<Option<Vec<u8>>>, i32, i32),
    UsePipeline(usize),
    BindMat(usize, Matrix4<f32>)
}

fn create_wgpu_pipeline(
    wm: &WmRenderer,
    attributes: &[SubmittedVertexAttrPointer],
    layout: &wgpu::PipelineLayout,
    shader: &dyn WmShader) -> wgpu::RenderPipeline {

    let mut shader_loc = 0;
    let layout_attrs: Vec<[wgpu::VertexAttribute; 1]> = attributes.iter().map(|attr| {
        shader_loc += 1;
        [wgpu::VertexAttribute {
            format: attr.format.as_wgpu(attr.size),
            offset: 0,
            shader_location: shader_loc - 1
        }; 1]
    }).collect();

    let mut index = 0;

    let buffers: Vec<wgpu::VertexBufferLayout> = attributes.iter().map(|attr| {
        index += 1;
        wgpu::VertexBufferLayout {
            array_stride: attr.stride as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &layout_attrs[index - 1]
        }
    }).collect();

    println!("Buffer layouts: {:?}\nLayout attrs: {:?}\n\n", buffers, layout_attrs);

    wm.wgpu_state.device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some(&format!("OpenGL pipeline ({:?}) with layout ({:?})", layout_attrs, layout)),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader.get_vert().0,
                entry_point: &shader.get_vert().1,
                buffers: &buffers
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader.get_frag().0,
                entry_point: shader.get_frag().1,
                targets: &[
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER
                        }),
                        write_mask: Default::default()
                    }
                ]
            }),
            multiview: None
        }
    )
}

fn tex_image_2d(wm: &WmRenderer, width: u32, height: u32, format: wgpu::TextureFormat, data: &[u8]) -> BindableTexture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1
    };

    let texture = wm.wgpu_state.device.create_texture(
        &wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        }
    );

    wm.wgpu_state.queue.write_texture(
        texture.as_image_copy(),
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(width as u32 * 4),
            rows_per_image: NonZeroU32::new(height as u32)
        },
        size
    );

    let view = texture.create_view(
        &wgpu::TextureViewDescriptor::default()
    );

    let sampler = wm.wgpu_state.device.create_sampler(
        &wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    );

    let bind_group = wm.wgpu_state.device.create_bind_group(
        &BindGroupDescriptor {
            label: None,
            layout: &wm.render_pipeline_manager.load().bind_group_layouts.read().get("texture").unwrap(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ]
        }
    );

    BindableTexture {
        tsv: TextureSamplerView {
            texture,
            view,
            sampler
        },
        bind_group
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SubmittedVertexAttrPointer {
    usage: GlAttributeType,
    format: GlAttributeFormat,
    size: u8,
    ptr: *const u8,
    stride: u32
}

fn quads_to_tris_transformer(vertex_data: &[u8], vertex_count: usize, stride: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(vertex_count * stride);
    for x in 0..vertex_count / 4 {
        let beginning_offset = (x * 4 * stride);
        let a = &vertex_data[beginning_offset..beginning_offset + stride];
        let b = &vertex_data[beginning_offset + stride..beginning_offset + (stride * 2)];
        let c = &vertex_data[beginning_offset + (stride * 2)..beginning_offset + (stride * 3)];
        let d = &vertex_data[beginning_offset + (stride * 3)..beginning_offset + (stride * 4)];
        // out.extend(a);
        // out.extend(b);
        // out.extend(c);
        // out.extend(a);
        // out.extend(c);
        // out.extend(d);
        out.extend(a);
        out.extend(c);
        out.extend(b);
        out.extend(a);
        out.extend(d);
        out.extend(c);
    }
    out
}

#[derive(Debug)]
pub struct GlPipeline {
    pub matrix_stacks: RefCell<[([Matrix4<f32>; 32], usize); 3]>,
    pub matrix_mode: RefCell<usize>,
    pub commands: ArcSwap<Vec<GLCommand>>,
    pub active_texture_slot: RefCell<i32>,
    pub slots: RefCell<HashMap<i32, i32>>,
    // pub vertex_attributes: RefCell<HashMap<GlAttributeType, SubmittedVertexAttrPointer>>,
    pub vertex_array: RefCell<Option<i32>>,
    pub client_states: RefCell<Vec<u32>>,
    pub active_pipeline: RefCell<usize>,
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
                Box::new(WgslShader::init(
                    &("wgpu_mc", "shaders/gui_col_pos.wgsl").into(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into()
                )) as Box<dyn WmShader>
            ),
            (
                "wgpu_mc_ogl:shaders/pos_col_uint".into(),
                Box::new(WgslShader::init(
                    &("wgpu_mc", "shaders/gui_col_pos_uint.wgsl").into(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into()
                )) as Box<dyn WmShader>
            ),
            (
                "wgpu_mc_ogl:shaders/pos_tex".into(),
                Box::new(WgslShader::init(
                    &("wgpu_mc", "shaders/gui_uv_pos.wgsl").into(),
                    &*wm.mc.resource_provider,
                    &wm.wgpu_state.device,
                    "fs_main".into(),
                    "vs_main".into()
                )) as Box<dyn WmShader>
            )
        ].into_iter().collect()
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
                wm.wgpu_state.device.create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            &layouts.get("matrix4").unwrap()
                        ],
                        push_constant_ranges: &[]
                    }
                )
            ),
            (
                "wgpu_mc_ogl:layouts/pos_tex".into(),
                wm.wgpu_state.device.create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            &layouts.get("matrix4").unwrap(),
                            &layouts.get("texture").unwrap()
                        ],
                        push_constant_ranges: &[]
                    }
                )
            )
        ].into_iter().collect()
    }

    fn build_wgpu_pipelines(&self, wm: &WmRenderer) -> HashMap<String, RenderPipeline> {
        let pipeline_manager = wm.render_pipeline_manager.load();
        let layouts = pipeline_manager.pipeline_layouts.load();
        let shaders = pipeline_manager.shader_map.read();

        let pos_col_float3_shader = shaders.get("wgpu_mc_ogl:shaders/pos_col_float3").unwrap();
        let pos_col_uint_shader = shaders.get("wgpu_mc_ogl:shaders/pos_col_uint").unwrap();
        let pos_tex_shader = shaders.get("wgpu_mc_ogl:shaders/pos_tex").unwrap();

        [
            (
                "wgpu_mc_ogl:pipelines/pos_col_float3".into(),
                wm.wgpu_state.device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&layouts.get("wgpu_mc_ogl:layouts/pos_col").unwrap()),
                        vertex: VertexState {
                            module: &pos_col_float3_shader.get_vert().0,
                            entry_point: &pos_col_float3_shader.get_vert().1,
                            buffers: &[
                                wgpu::VertexBufferLayout {
                                    array_stride: 24,
                                    step_mode: wgpu::VertexStepMode::Vertex,
                                    attributes: &[
                                        wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x3,
                                            offset: 0,
                                            shader_location: 0
                                        },
                                        wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x3,
                                            offset: 12,
                                            shader_location: 1
                                        }
                                    ]
                                }
                            ]
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false
                        },
                        depth_stencil: Some(
                            wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: false,
                                depth_compare: wgpu::CompareFunction::Always,
                                stencil: Default::default(),
                                bias: Default::default()
                            }
                        ),
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: &pos_col_float3_shader.get_frag().0,
                            entry_point: &pos_col_float3_shader.get_frag().1,
                            targets: &[
                                wgpu::ColorTargetState {
                                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                    blend: None,
                                    write_mask: Default::default()
                                }
                            ]
                        }),
                        multiview: None
                    }
                )
            ),
            (
                "pos_tex".into(),
                wm.wgpu_state.device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&layouts.get("wgpu_mc_ogl:layouts/pos_tex").unwrap()),
                        vertex: VertexState {
                            module: &pos_tex_shader.get_vert().0,
                            entry_point: &pos_tex_shader.get_vert().1,
                            buffers: &[
                                wgpu::VertexBufferLayout {
                                    array_stride: 20,
                                    step_mode: wgpu::VertexStepMode::Vertex,
                                    attributes: &[
                                        wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x3,
                                            offset: 0,
                                            shader_location: 0
                                        },
                                        wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x2,
                                            offset: 12,
                                            shader_location: 1
                                        }
                                    ]
                                }
                            ]
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false
                        },
                        depth_stencil: Some(
                            wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: false,
                                depth_compare: wgpu::CompareFunction::Always,
                                stencil: Default::default(),
                                bias: Default::default()
                            }
                        ),
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: &pos_tex_shader.get_frag().0,
                            entry_point: &pos_tex_shader.get_frag().1,
                            targets: &[
                                wgpu::ColorTargetState {
                                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                    blend: None,
                                    write_mask: Default::default()
                                }
                            ]
                        }),
                        multiview: None
                    }
                )
            ),
            (
                "pos_col_uint".into(),
                wm.wgpu_state.device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&layouts.get("wgpu_mc_ogl:layouts/pos_col").unwrap()),
                        vertex: VertexState {
                            module: &pos_col_uint_shader.get_vert().0,
                            entry_point: &pos_col_uint_shader.get_vert().1,
                            buffers: &[
                                wgpu::VertexBufferLayout {
                                    array_stride: 16,
                                    step_mode: wgpu::VertexStepMode::Vertex,
                                    attributes: &[
                                        wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x3,
                                            offset: 0,
                                            shader_location: 0
                                        },
                                        wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Uint32,
                                            offset: 12,
                                            shader_location: 1
                                        }
                                    ]
                                }
                            ]
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false
                        },
                        depth_stencil: Some(
                            wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: false,
                                depth_compare: wgpu::CompareFunction::Always,
                                stencil: Default::default(),
                                bias: Default::default()
                            }
                        ),
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: &pos_col_uint_shader.get_frag().0,
                            entry_point: &pos_col_uint_shader.get_frag().1,
                            targets: &[
                                wgpu::ColorTargetState {
                                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                    blend: None,
                                    write_mask: Default::default()
                                }
                            ]
                        }),
                        multiview: None
                    }
                )
            )
        ].into()
    }

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
        let pipeline_manager = renderer.render_pipeline_manager.load();
        let gl_alloc = unsafe { &GL_ALLOC }.get().unwrap();

        let commands = self.commands.load();

        commands.iter().for_each(|command| {
            match command {
                GLCommand::BindMat(slot, mat) => {
                    let helper = UniformMatrixHelper {
                        view_proj: (*mat).into()
                    };
                    let mat_buffer = arena.alloc(
                        renderer.wgpu_state.device.create_buffer_init(
                            &BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::bytes_of(&helper),
                                usage: wgpu::BufferUsages::UNIFORM
                            }
                        )
                    );
                    let group = arena.alloc(renderer.wgpu_state.device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &pipeline_manager.bind_group_layouts.read().get("matrix4").unwrap(),
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mat_buffer.as_entire_binding()
                                }
                            ]
                        }
                    ));

                    render_pass.set_bind_group(0, group, &[]);
                },
                GLCommand::UsePipeline(pipeline) => {
                    *self.active_pipeline.borrow_mut() = *pipeline;

                    render_pass.set_pipeline(
                        arena.alloc(match pipeline {
                            0 => pipeline_manager.render_pipelines.load().get("pos_col_uint").unwrap().clone(),
                            1 => pipeline_manager.render_pipelines.load().get("pos_tex").unwrap().clone(),
                            _ => unimplemented!()
                        })
                    )
                },
                GLCommand::BindVertexArray(array) => {
                    *self.vertex_array.borrow_mut() = Some(*array);
                    // let buf = unsafe { gl::get_buffer(*array as usize) }
                    //     .unwrap();
                    // render_pass.set_vertex_buffer(0, arena.alloc(buf).slice(..));
                },
                GLCommand::BindTexture(texture_id) => {
                    let mut slots = self.slots.borrow_mut();
                    slots.insert(*self.active_texture_slot.borrow(), *texture_id);
                },
                GLCommand::ActiveTexture(slot) => {
                    let mut active_slot = self.active_texture_slot.borrow_mut();
                    *active_slot = *slot;
                },
                GLCommand::BindBuffer(target, buffer_id) => {
                    let mut slots = self.slots.borrow_mut();
                    slots.insert(*target, *buffer_id);
                },
                GLCommand::DrawElements(mode, count, type_, indices) => {
                    let slots = self.slots.borrow();
                    // let vertex_array_id = self.vertex_array.borrow().unwrap();
                    let active_pipeline = *self.active_pipeline.borrow();
                    let textured = active_pipeline == 1;

                    let vertex_array = unsafe {
                        get_buffer(*slots.get(&0x8892i32).unwrap() as usize)
                    }.unwrap();

                    render_pass.set_vertex_buffer(0, arena.alloc(vertex_array).buffer.as_ref().unwrap().slice(..));

                    let mut gl_element_buffer = unsafe {
                        get_buffer(*slots.get(&0x8893i32).unwrap() as usize)
                    }.unwrap();

                    let mut index_buffer = arena.alloc(
                        gl_element_buffer.buffer.as_ref().unwrap().clone()
                    );

                    let (index_size, index_format) = match type_ {
                        0x1401 => {
                            let short_array = byte_buffer_to_short(
                                &gl_element_buffer.data.as_ref().unwrap()
                            );
                            index_buffer = arena.alloc(Rc::new(renderer.wgpu_state.device.create_buffer_init(
                                &BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice::<_, u8>(&short_array[..]),
                                    usage: wgpu::BufferUsages::INDEX
                                }
                            )));
                            (2, wgpu::IndexFormat::Uint16)
                        },
                        0x1403 => (2, wgpu::IndexFormat::Uint16),
                        0x1405 => (4, wgpu::IndexFormat::Uint32),
                        _ => panic!("Unknown elements type")
                    };

                    render_pass.set_index_buffer(index_buffer.slice(..), index_format);

                    if textured {
                        let active_tex_slot = *self.active_texture_slot.borrow();
                        let slots = self.slots.borrow();
                        let active_tex = *slots.get(&active_tex_slot).unwrap();
                        let texture = arena.alloc(
                            unsafe { gl::get_texture(active_tex as usize) }.unwrap()
                        );
                        render_pass.set_bind_group(1, &texture.bind_group, &[]);
                    }

                    render_pass.draw_indexed(0..*count as u32, 0, 0..1);
                },
                GLCommand::ClearColor(r, g, b) => {
                    let mut vec = Vec::with_capacity(96);
                    vec.extend((-1f32).to_ne_bytes());
                    vec.extend((-1f32).to_ne_bytes());
                    vec.extend(0f32.to_ne_bytes());
                    vec.extend(r.to_ne_bytes());
                    vec.extend(g.to_ne_bytes());
                    vec.extend(b.to_ne_bytes());

                    vec.extend(1f32.to_ne_bytes());
                    vec.extend((-1f32).to_ne_bytes());
                    vec.extend(0f32.to_ne_bytes());
                    vec.extend(r.to_ne_bytes());
                    vec.extend(g.to_ne_bytes());
                    vec.extend(b.to_ne_bytes());

                    vec.extend(1f32.to_ne_bytes());
                    vec.extend(1f32.to_ne_bytes());
                    vec.extend(0f32.to_ne_bytes());
                    vec.extend(r.to_ne_bytes());
                    vec.extend(g.to_ne_bytes());
                    vec.extend(b.to_ne_bytes());

                    vec.extend((-1f32).to_ne_bytes());
                    vec.extend(1f32.to_ne_bytes());
                    vec.extend(0f32.to_ne_bytes());
                    vec.extend(r.to_ne_bytes());
                    vec.extend(g.to_ne_bytes());
                    vec.extend(b.to_ne_bytes());

                    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
                    let indices_buffer = arena.alloc(renderer.wgpu_state.device.create_buffer_init(
                        &BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::bytes_of(&indices),
                            usage: wgpu::BufferUsages::INDEX
                        }
                    ));

                    let vertex_buffer = arena.alloc(renderer.wgpu_state.device.create_buffer_init(
                        &BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::bytes_of(&indices),
                            usage: wgpu::BufferUsages::VERTEX
                        }
                    ));

                    let helper = UniformMatrixHelper {
                        view_proj: (Matrix4::identity()).into()
                    };
                    let mat_buffer = arena.alloc(
                        renderer.wgpu_state.device.create_buffer_init(
                            &BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::bytes_of(&helper),
                                usage: wgpu::BufferUsages::UNIFORM
                            }
                        )
                    );
                    let group = arena.alloc(renderer.wgpu_state.device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &pipeline_manager.bind_group_layouts.read().get("matrix4").unwrap(),
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mat_buffer.as_entire_binding()
                                }
                            ]
                        }
                    ));

                    render_pass.set_bind_group(0, group, &[]);

                    render_pass.set_pipeline(arena.alloc(pipeline_manager.render_pipelines.load().get("wgpu_mc_ogl:pipelines/pos_col_float3").unwrap().clone()));
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..1);
                },
                GLCommand::PushMatrix => {
                    let mut stacks = self.matrix_stacks.borrow_mut();
                    let mode = *self.matrix_mode.borrow();
                    let mut active_stack_tuple = &mut stacks[mode];
                    //Duplicate the current stack
                    active_stack_tuple.0[active_stack_tuple.1 + 1] = active_stack_tuple.0[active_stack_tuple.1];
                    //Increment the stack offset
                    active_stack_tuple.1 += 1;
                }
                GLCommand::PopMatrix => {
                    let mut stacks = self.matrix_stacks.borrow_mut();
                    let mode = *self.matrix_mode.borrow();
                    let mut active_stack_tuple = &mut stacks[mode];
                    //Decrement the stack offset
                    active_stack_tuple.1 -= 1;
                }
                GLCommand::TexImage2D(command) => {
                    if command.0.borrow().is_some() {
                        let mut slots = self.slots.borrow_mut();
                        let active_slot = *self.active_texture_slot.borrow();
                        let active_texture_id = *slots.get(&active_slot).unwrap();
                        let slab = unsafe { &mut GL_ALLOC }.get_mut().unwrap();
                        let resource = slab.get_mut(active_texture_id as usize).unwrap();
                        let material =
                            tex_image_2d(
                                renderer,
                                command.1,
                                command.2,
                                command.3,
                                &command.0.borrow_mut().take().unwrap());

                        match resource {
                            GlResource::Texture(tex) => {
                                tex.material = Some(
                                    Rc::new(material)
                                )
                            }
                            GlResource::Buffer(_) => panic!("Invalid command")
                        };
                    }
                },
                // GLCommand::VertexPointer(size, format, stride, pointer) => {
                //     let mut attrs = self.vertex_attributes.borrow_mut();
                //
                //     attrs.insert(
                //         GlAttributeType::Position,
                //         SubmittedVertexAttrPointer {
                //             usage: GlAttributeType::Position,
                //             format: GlAttributeFormat::from_enum(*format as u32),
                //             size: *size as u8,
                //             ptr: *pointer,
                //             stride: *stride as u32
                //         }
                //     );
                // }
                // GLCommand::ColorPointer(size, format, stride, pointer) => {
                //     let mut attrs = self.vertex_attributes.borrow_mut();
                //
                //     attrs.insert(
                //         GlAttributeType::Color,
                //         SubmittedVertexAttrPointer {
                //             usage: GlAttributeType::Color,
                //             format: GlAttributeFormat::from_enum(*format as u32),
                //             size: *size as u8,
                //             ptr: *pointer,
                //             stride: *stride as u32
                //         }
                //     );
                // }
                // GLCommand::TexCoordPointer(size, format, stride, pointer) => {
                //     let mut attrs = self.vertex_attributes.borrow_mut();
                //
                //     attrs.insert(
                //         GlAttributeType::UV,
                //         SubmittedVertexAttrPointer {
                //             usage: GlAttributeType::UV,
                //             format: GlAttributeFormat::from_enum(*format as u32),
                //             size: *size as u8,
                //             ptr: *pointer,
                //             stride: *stride as u32
                //         }
                //     );
                // }
                // GLCommand::EnableClientState(state) => {
                //     self.client_states.borrow_mut().push(*state);
                // }
                // GLCommand::DisableClientState(state) => {
                //     let mut states = self.client_states.borrow_mut();
                //     *states = states.iter().copied().filter(|&client_state| client_state != *state).collect();
                // },
                GLCommand::MultMatrix(mat) => {
                    let mode = *self.matrix_mode.borrow();
                    let mut stacks = self.matrix_stacks.borrow_mut();
                    let mut stack = &mut stacks[mode];
                    stack.0[stack.1 as usize] = stack.0[stack.1 as usize] * mat;
                },
                GLCommand::SetMatrix(mat) => {
                    let mode = *self.matrix_mode.borrow();
                    let mut stacks = self.matrix_stacks.borrow_mut();
                    let mut stack = &mut stacks[mode];
                    stack.0[stack.1 as usize] = *mat;
                },
                GLCommand::MatrixMode(mode) => {
                    *self.matrix_mode.borrow_mut() = mode - 0x1700;
                }
                // GLCommand::Ortho()
                _ => panic!("Unimplemented GL command")
            };
        });
    }
}