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
use crate::gl::{GL_ALLOC, GlResource, GlAttributeType, GlVertexAttribute, GlAttributeFormat, get_texture};
use std::collections::HashMap;
use std::rc::Rc;
use std::num::NonZeroU32;
use wgpu_mc::mc::datapack::NamespacedResource;
use wgpu_mc::render::shader::{Shader, ShaderSource};
use futures::StreamExt;
use wgpu_mc::util::WmArena;

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

pub fn create_wgpu_pipeline_layout(wm: &WmRenderer, tex_bg: bool) -> wgpu::PipelineLayout {
    let pipelines = wm.pipelines.load();
    let mut layouts = vec![
        &pipelines.layouts.matrix_bind_group_layout
    ];

    if tex_bg {
        layouts.push(&pipelines.layouts.texture_bind_group_layout);
    }

    wm.wgpu_state.device.create_pipeline_layout(
        &PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[]
        }
    )
}

fn create_wgpu_pipeline(
    wm: &WmRenderer,
    attributes: &[SubmittedVertexAttrPointer],
    layout: &wgpu::PipelineLayout,
    shader: &Shader) -> wgpu::RenderPipeline {
    println!("Creating wgpu pipeline for vertex layout {:?}", attributes);

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
                unclipped_depth: false,
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
            }),
            multiview: None
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

#[derive(Debug, Copy, Clone)]
pub struct SubmittedVertexAttrPointer {
    usage: GlAttributeType,
    format: GlAttributeFormat,
    size: u8,
    ptr: u64,
    stride: u32
}

pub struct GlPipeline {
    pub pipelines: RefCell<HashMap<Vec<GlAttributeType>, Rc<wgpu::RenderPipeline>>>,
    //Probably a fine amount
    pub matrix_stack: RefCell<[Matrix4<f32>; 32]>,
    pub matrix_offset: RefCell<i8>,
    pub commands: ArcSwap<Vec<GLCommand>>,
    pub active_texture_slot: RefCell<i32>,
    pub slots: RefCell<HashMap<i32, i32>>,
    pub vertex_attributes: RefCell<HashMap<GlAttributeType, SubmittedVertexAttrPointer>>,
    pub client_states: RefCell<Vec<u32>>,
    pub shaders: RefCell<Option<(Shader, Shader)>>
}

impl WmPipeline for GlPipeline {
    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
        let wm_pipelines = arena.alloc(renderer.pipelines.load_full());
        let sc = renderer.surface_config.load();
        let gl_alloc = unsafe { GL_ALLOC.assume_init_ref() };

        let mut shaders = self.shaders.borrow_mut();

        if shaders.is_none() {
            let mut compiler = renderer.shaderc.lock();
            let resource_provider = &renderer.mc.resource_provider;

            let pos_col = Shader::from_glsl(
                ShaderSource {
                    file_name: "gui_col_pos.fsh",
                    source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/gui_col_pos.fsh").into())).unwrap(),
                    entry_point: "main"
                },
                ShaderSource {
                    file_name: "gui_col_pos.vsh",
                    source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/gui_col_pos.vsh").into())).unwrap(),
                    entry_point: "main"
                },
                &renderer.wgpu_state.device,
                &mut compiler
            ).unwrap();

            let pos_uv = Shader::from_glsl(
                ShaderSource {
                    file_name: "gui_uv_pos.fsh",
                    source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/gui_uv_pos.fsh").into())).unwrap(),
                    entry_point: "main"
                },
                ShaderSource {
                    file_name: "gui_uv_pos.vsh",
                    source: std::str::from_utf8(&resource_provider.get_resource(&("wgpu_mc", "shaders/gui_uv_pos.vsh").into())).unwrap(),
                    entry_point: "main"
                },
                &renderer.wgpu_state.device,
                &mut compiler
            ).unwrap();

            *shaders = Some((pos_col, pos_uv));
        }

        let commands = self.commands.load();
        commands.iter().for_each(|command| {
            match command {
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
                }
                GLCommand::DrawArray(mode, first, count) => {
                    let device = &renderer.wgpu_state.device;
                    let vertex_attributes = self.vertex_attributes.borrow_mut();
                    let client_states = self.client_states.borrow();

                    let enabled_attributes: Vec<&SubmittedVertexAttrPointer> = client_states.iter()
                        .filter_map(|state| {
                            match state {
                                0x8076 => vertex_attributes
                                        .iter()
                                        .find(
                                            |(&kind,
                                                 attr
                                             )| kind == GlAttributeType::Color),
                                0x8074 => vertex_attributes
                                        .iter()
                                        .find(
                                            |(&kind,
                                                 attr
                                             )| kind == GlAttributeType::Position),
                                0x8078 => vertex_attributes
                                        .iter()
                                        .find(
                                            |(&kind,
                                                 attr
                                             )| kind == GlAttributeType::UV),
                                _ => None
                            }
                        })
                        .map(|tuple| tuple.1)
                        .collect();

                    enabled_attributes.iter().map(|attr| {
                        let slice = unsafe {
                            std::slice::from_raw_parts(
                                attr.ptr as *mut u8,
                                attr.stride as usize * *count as usize
                            )
                        };

                        // println!("{:?} {:?}", attr, slice);

                        arena.alloc(device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: slice,
                            usage: wgpu::BufferUsages::VERTEX
                        }))
                    }).enumerate().for_each(|(index, buf)| {
                        render_pass.set_vertex_buffer(index as u32, buf.slice(..))
                    });

                    let mut pipelines = self.pipelines.borrow_mut();

                    let pipeline_key: Vec<GlAttributeType> = enabled_attributes.iter()
                        .map(|ptr| ptr.usage)
                        .collect();

                    let needs_tex = enabled_attributes[1].usage == GlAttributeType::UV;

                    if !pipelines.contains_key(&pipeline_key) {
                        let layout = create_wgpu_pipeline_layout(renderer, needs_tex);
                        println!("Created layout");
                        let attributes: Vec<SubmittedVertexAttrPointer> = enabled_attributes.iter().copied().copied().collect();
                        let new_pipeline = create_wgpu_pipeline(
                            renderer,
                            &attributes,
                            &layout,
                            if needs_tex {
                                &shaders.as_ref().unwrap().0
                            } else {
                                &shaders.as_ref().unwrap().1
                            }
                        );
                        pipelines.insert(pipeline_key.clone(), Rc::new(new_pipeline));
                    }

                    let render_pipeline = pipelines.get(&pipeline_key)
                        .unwrap()
                        .clone();

                    let matrix_stack = self.matrix_stack.borrow();
                    let matrix = UniformMatrixHelper {
                        view_proj: matrix_stack[*self.matrix_offset.borrow() as usize].into()
                    };

                    let buffer = device.create_buffer_init(
                        &BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::bytes_of(&matrix),
                            usage: wgpu::BufferUsages::UNIFORM
                        }
                    );
                    let matrix_uploaded = arena.alloc(buffer);
                    let matrix_bind_group = device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &wm_pipelines.layouts.matrix_bind_group_layout,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: matrix_uploaded.as_entire_binding()
                                }
                            ]
                        }
                    );

                    render_pass.set_bind_group(0, arena.alloc(matrix_bind_group), &[]);
                    if needs_tex {
                        let active_slot = *self.active_texture_slot.borrow();
                        let bound_texture = *self.slots.borrow().get(&active_slot).unwrap();
                        let texture = unsafe { get_texture(bound_texture as usize) };
                        render_pass.set_bind_group(1, &texture.unwrap().bind_group, &[]);
                    }

                    render_pass.set_pipeline(arena.alloc(render_pipeline));

                    render_pass.draw(*first as u32..*first as u32+ *count as u32, 0..1);
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
                        let active_slot = *self.active_texture_slot.borrow();
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

                    attrs.insert(
                        GlAttributeType::Position,
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

                    attrs.insert(
                        GlAttributeType::Color,
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

                    attrs.insert(
                        GlAttributeType::UV,
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
                    *states = states.iter().copied().filter(|&client_state| client_state != *state).collect();
                }
                _ => {}
            };
        });
    }
}