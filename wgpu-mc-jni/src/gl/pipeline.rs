use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::WmRenderer;
use wgpu::{RenderPass, BufferDescriptor, BufferUsages, BindGroupDescriptor, BindGroupEntry, BindGroup, PipelineLayoutDescriptor, PipelineLayout, RenderPipeline, VertexState, PrimitiveState, FrontFace, ShaderModuleDescriptor};
use wgpu_mc::texture::{UV, WgpuTexture};
use cgmath::{Matrix2, Matrix3, Matrix4, Vector3};
use wgpu_mc::camera::UniformMatrixHelper;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use arc_swap::ArcSwap;
use wgpu_mc::model::Material;
use std::cell::{RefCell, Cell};
use crate::gl::{GL_ALLOC, GlResource, GlAttributeType, GlVertexAttribute, GlAttributeFormat};
use std::collections::HashMap;
use std::rc::Rc;
use std::num::NonZeroU32;
use wgpu_mc::mc::datapack::NamespacedResource;
use wgpu_mc::render::shader::Shader;

#[derive(Clone, Debug)]
pub enum GLCommand {
    BindTexture(i32),
    BindBuffer(i32, i32),
    ActiveTexture(i32),
    DrawArray(i32, i32, i32),
    PushMatrix,
    PopMatrix,
    VertexPointer(i32, i32, i32, u64),
    ColorPointer(i32, i32, i32, u64),
    TexCoordPointer(i32, i32, i32, u64),
    TexImage2D((RefCell<Option<Vec<u8>>>, u32, u32, wgpu::TextureFormat)),
    EnableClientState(u32),
    DisableClientState(u32)
}

pub fn create_wgpu_pipeline_layout(wm: &WmRenderer) -> wgpu::PipelineLayout {
    // let mut vertex = GlVertex {
    //     gl_attributes: vec![],
    //     wgpu_attributes: vec![]
    // };
    // for offset in 0..8 {
    //     let count = (vertex_counts >> (offset * 8)) as u8;
    //     let types_byte = (vertex_types >> (offset * 8)) as u8;
    //     let format_byte = (vertex_format >> (offset * 8)) as u8;
    //     let attr_type = match types_byte {
    //         0 => break,
    //         0b1 => GlAttributeType::Position,
    //         0b10 => GlAttributeType::Color,
    //         0b100 => unimplemented!(),
    //         0b1000 => GlAttributeType::Normal,
    //         0b10000 => GlAttributeType::UV,
    //         _ => panic!("Invalid packed attribute")
    //     };
    //     let format = match format_byte {
    //         0b1 => GlAttributeFormat::Byte,
    //         0b10 => GlAttributeFormat::Float,
    //         0b100 => GlAttributeFormat::Int,
    //         0b1000 => unimplemented!(),
    //         0b10000 => GlAttributeFormat::UByte,
    //         0b100000 => GlAttributeFormat::Int,
    //         0b1000000 => GlAttributeFormat::Short,
    //         _ => panic!("Invalid packed attribute")
    //     };
    //     vertex.gl_attributes.push(GlVertexAttribute {
    //         count,
    //         format,
    //         attr_type,
    //         stride: 0
    //     });
    // }
    wm.wgpu_state.device.create_pipeline_layout(
        &PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &wm.pipelines.load().layouts.matrix_bind_group_layout,
                &wm.pipelines.load().layouts.texture_bind_group_layout
            ],
            push_constant_ranges: &[]
        }
    )
}

pub fn create_wgpu_pipeline(
    wm: &WmRenderer,
    attributes: &[SubmittedVertexAttrPointer],
    layout: &wgpu::PipelineLayout,
    shader: &Shader) -> wgpu::RenderPipeline {

    let mut shader_loc = 0;
    let buffers: Vec<wgpu::VertexBufferLayout> = attributes.iter().map(|attr| {
        shader_loc += 1;
        wgpu::VertexBufferLayout {
            array_stride: attr.stride as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: attr.format.as_wgpu(attr.size),
                    offset: 0,
                    shader_location: shader_loc - 1
                }
            ]
        }
    }).collect();

    wm.wgpu_state.device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader.vert,
                entry_point: "main",
                buffers: &buffers
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader.frag,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE
                        }),
                        write_mask: Default::default()
                    }
                ]
            })
        }
    )
}

fn tex_image_2d(wm: &WmRenderer, width: u32, height: u32, format: wgpu::TextureFormat, data: &[u8]) -> Material {
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
            layout: &wm.pipelines.load().layouts.texture_bind_group_layout,
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

    Material {
        name: Arc::new("".to_string()),
        diffuse_texture: WgpuTexture {
            texture,
            view,
            sampler
        },
        bind_group
    }
}

struct SubmittedVertexAttrPointer {
    usage: GlAttributeType,
    format: GlAttributeFormat,
    size: u8,
    ptr: u64,
    stride: u32
}

pub struct GlPipeline {
    pub pipelines: HashMap<u64, wgpu::RenderPipeline>,
    //Probably a fine amount
    pub matrix_stack: RefCell<[Matrix4<f32>; 32]>,
    pub matrix_offset: RefCell<i8>,
    pub commands: ArcSwap<Vec<GLCommand>>,
    pub active_slot: RefCell<i32>,
    pub slots: RefCell<HashMap<i32, i32>>,
    pub vertex_attributes: RefCell<Vec<SubmittedVertexAttrPointer>>,
    pub client_states: RefCell<Vec<u32>>
}

impl WmPipeline for GlPipeline {
    fn render<'a, 'b, 'c, 'd: 'c, 'e: 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'e bumpalo::Bump) {
        let pipelines = arena.alloc(renderer.pipelines.load_full());
        let sc = renderer.surface_config.load();
        let gl_alloc = unsafe { GL_ALLOC.assume_init_ref() };

        // render_pass.set_pipeline(&gl_pipeline);

        let commands = self.commands.load();
        commands.iter().for_each(|command| {
            match command {
                GLCommand::BindTexture(texture_id) => {
                    let mut slots = self.slots.borrow_mut();
                    slots.insert(*self.active_slot.borrow(), *texture_id);
                },
                GLCommand::ActiveTexture(slot) => {
                    let mut active_slot = self.active_slot.borrow_mut();
                    *active_slot = *slot;
                },
                GLCommand::BindBuffer(target, buffer_id) => {
                    let mut slots = self.slots.borrow_mut();
                    slots.insert(*target, *buffer_id);
                }
                GLCommand::DrawArray(mode, first, count) => {
                    // let vertex_attributes = self.vertex_attributes
                    // let slots = self.slots.borrow();
                    // let gl_array_buffer = *slots.get(&0x8892).unwrap();
                    // match gl_alloc.get(gl_array_buffer as usize).unwrap() {
                    //     GlResource::Texture(texture) => panic!("Invalid command"),
                    //     GlResource::Buffer(buffer) => {
                    //         render_pass.set_vertex_buffer(
                    //             0,
                    //             arena.alloc(buffer.buffer.as_ref().unwrap().clone()).slice(..)
                    //         );
                    //         render_pass.draw(*first as u32..*first as u32 + *count as u32, 0..1);
                    //     }
                    // }
                }
                GLCommand::PushMatrix => {
                    let current_offset = *self.matrix_offset.borrow();
                    let new_offset = current_offset + 1;
                    let mut stack = self.matrix_stack.borrow_mut();
                    stack[new_offset as usize] = stack[current_offset as usize];
                }
                GLCommand::PopMatrix => {
                    self.matrix_offset.borrow_mut().checked_add(-1).unwrap();
                }
                GLCommand::TexImage2D(command) => {
                    if command.0.borrow().is_some() {
                        let mut slots = self.slots.borrow_mut();
                        let active_slot = *self.active_slot.borrow();
                        let active_texture_id = *slots.get(&active_slot).unwrap();
                        let slab = unsafe { GL_ALLOC.assume_init_mut() };
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
                                    material
                                )
                            }
                            GlResource::Buffer(_) => panic!("Invalid command")
                        };
                    }
                }
                GLCommand::VertexPointer(size, format, stride, pointer) => {
                    let mut attrs = self.vertex_attributes.borrow_mut();

                    attrs.push(
                        SubmittedVertexAttrPointer {
                            usage: GlAttributeType::Position,
                            format: GlAttributeFormat::from_enum(*format as u32),
                            size: *size as u8,
                            ptr: *pointer,
                            stride: *stride as u32
                        }
                    );
                }
                GLCommand::ColorPointer(size, format, stride, pointer) => {
                    let mut attrs = self.vertex_attributes.borrow_mut();

                    attrs.push(
                        SubmittedVertexAttrPointer {
                            usage: GlAttributeType::Color,
                            format: GlAttributeFormat::from_enum(*format as u32),
                            size: *size as u8,
                            ptr: *pointer,
                            stride: *stride as u32
                        }
                    );
                }
                GLCommand::TexCoordPointer(size, format, stride, pointer) => {
                    let mut attrs = self.vertex_attributes.borrow_mut();

                    attrs.push(
                        SubmittedVertexAttrPointer {
                            usage: GlAttributeType::UV,
                            format: GlAttributeFormat::from_enum(*format as u32),
                            size: *size as u8,
                            ptr: *pointer,
                            stride: *stride as u32
                        }
                    );
                }
                GLCommand::EnableClientState(state) => {
                    self.client_states.borrow_mut().push(*state);
                }
                GLCommand::DisableClientState(state) => {
                    let mut states = self.client_states.borrow_mut();
                    *states = states.iter().filter(|&client_state| client_state != state).collect();
                }
            };
        });
    }
}