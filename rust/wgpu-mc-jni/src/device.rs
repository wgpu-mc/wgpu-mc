use crate::glfw::LWJGLGLFWWindow;
use crate::{MinecraftResourceManagerAdapter, RENDERER};
use futures::executor::block_on;
use jni::objects::{JByteBuffer, JClass, JString};
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{
    HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, CStr, CString};
use std::num::{NonZeroIsize, NonZeroU64};
use std::ops::Range;
use std::sync::Arc;
use wgpu_mc::texture::TextureAndView;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_mc::wgpu::{BufferUsages, TextureUsages, TextureViewDescriptor};
use wgpu_mc::{wgpu, Display, WindowSize, WmRenderer};

struct BuiltinPipelines {
    blit: wgpu::RenderPipeline,
    gui_textured: wgpu::RenderPipeline,
}

static BUILTIN_PIPELINES: OnceCell<BuiltinPipelines> = OnceCell::new();

#[repr(C)]
struct McRenderPass<'a> {
    target: &'a TextureAndView,
    depth_target: Option<&'a TextureAndView>,
    commands: *mut RenderPassCommand<'a>,
    commands_len: u32,
}

#[repr(u32)]
#[derive(Debug)]
enum IndexType {
    I16 = 0,
    I32 = 1,
}

#[repr(u64)]
#[derive(Debug)]
enum RenderPassCommand<'a> {
    Draw {
        offset: u32,
        count: u32,
    } = 0,
    DrawIndexed {
        offset: u32,
        count: u32,
        primcount: u32,
        i: i32,
    } = 1,
    SetIndexBuffer {
        index_buffer: &'a wgpu::Buffer,
        index_type: IndexType,
    } = 2,
    SetVertexBuffer {
        vertex_buffer: &'a wgpu::Buffer,
        index: u32,
    } = 3,
    SetPipeline(u32) = 4,
    BindTexture {
        texture: &'a TextureAndView,
        name: *mut u8,
        name_len: u32,
    } = 5,
    BindBuffer {
        buffer: &'a wgpu::Buffer,
        name: *mut u8,
        name_len: u32,
        start: u32,
        end: u32,
    } = 6,
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getRenderPassCommandSize(env: JNIEnv, _class: JClass) -> jlong {
    size_of::<RenderPassCommand>() as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createDevice(
    env: JNIEnv,
    _class: JClass,
    window: jlong,
    native_window: jlong,
    width: jint,
    height: jint,
) {
    let width = width as u32;
    let height = height as u32;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let handle = unsafe { LWJGLGLFWWindow::new(native_window as _) };

    let surface = instance.create_surface(Arc::new(handle)).unwrap();

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    const VSYNC: bool = false;

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width,
        height,
        present_mode: if VSYNC {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        },

        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };

    let required_limits = wgpu::Limits {
        max_push_constant_size: 128,
        max_bind_groups: 8,
        ..Default::default()
    };

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::default()
                | wgpu::Features::DEPTH_CLIP_CONTROL
                | wgpu::Features::PUSH_CONSTANTS
                // | wgpu::Features::BUFFER_BINDING_ARRAY
                | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                // | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
                | wgpu::Features::MULTI_DRAW_INDIRECT,
            required_limits,
            memory_hints: wgpu::MemoryHints::Performance,
        },
        None, // Trace path
    ))
    .unwrap();

    surface.configure(&device, &surface_config);

    let display = Display {
        surface,
        device,
        queue,
        config: RwLock::new(surface_config),
        instance,
        adapter,
    };

    let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
        jvm: env.get_java_vm().unwrap(),
    });

    let wm = WmRenderer::new(display, resource_provider);

    wm.init();

    let blit_shader = wm
        .gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../blit.wgsl").into()),
        });

    let gui_textured_shader = wm
        .gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../gui_textured.wgsl").into()),
        });

    let gui_textured_pipeline_layout =
        wm.gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    wm.bind_group_layouts.get("texture_sampler").unwrap(),
                    wm.bind_group_layouts.get("ssbo").unwrap(),
                    wm.bind_group_layouts.get("ssbo").unwrap(),
                ],
                push_constant_ranges: &[],
            });

    let blit_pipeline_layout =
        wm.gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[wm.bind_group_layouts.get("texture_sampler").unwrap()],
                push_constant_ranges: &[],
            });

    let blit_pipeline = wm
        .gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "frag",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview: None,
            cache: None,
        });

    let gui_textured_pipeline =
        wm.gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&gui_textured_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &gui_textured_shader,
                    entry_point: "vert",
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 24,
                        step_mode: Default::default(),
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3,
                            1 => Float32x2,
                            2 => Uint32,
                        ],
                    }],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &gui_textured_shader,
                    entry_point: "frag",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            });

    drop(BUILTIN_PIPELINES.set(BuiltinPipelines {
        blit: blit_pipeline,
        gui_textured: gui_textured_pipeline,
    }));

    drop(RENDERER.set(wm));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createBuffer(
    mut env: JNIEnv,
    _class: JClass,
    label: JString,
    usage: jint,
    size: jint,
) -> jlong {
    let wm = RENDERER.get().unwrap();

    let label = env.get_string(&label).unwrap();

    let mut wgpu_usage_flags = BufferUsages::empty();
    wgpu_usage_flags.set(BufferUsages::MAP_READ, usage & 1 != 0);
    wgpu_usage_flags.set(BufferUsages::MAP_WRITE, usage & 2 != 0);
    wgpu_usage_flags.set(BufferUsages::COPY_DST, usage & 8 != 0);
    wgpu_usage_flags.set(BufferUsages::COPY_SRC, usage & 16 != 0);
    wgpu_usage_flags.set(BufferUsages::VERTEX, usage & 32 != 0);
    wgpu_usage_flags.set(BufferUsages::INDEX, usage & 64 != 0);
    wgpu_usage_flags.set(BufferUsages::STORAGE, usage & 128 != 0);

    let buffer = wm.gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label.to_str().unwrap()),
        size: size as _,
        usage: wgpu_usage_flags | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    Box::into_raw(Box::new(buffer)) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createBufferInit(
    mut env: JNIEnv,
    _class: JClass,
    label: JString,
    usage: jint,
    data: JByteBuffer,
) -> jlong {
    let wm = RENDERER.get().unwrap();

    let label = env.get_string(&label).unwrap();
    let data = unsafe {
        std::slice::from_raw_parts(
            env.get_direct_buffer_address(&data).unwrap(),
            env.get_direct_buffer_capacity(&data).unwrap(),
        )
    };

    let mut wgpu_usage_flags = BufferUsages::empty();
    wgpu_usage_flags.set(BufferUsages::MAP_READ, usage & 1 != 0);
    wgpu_usage_flags.set(BufferUsages::MAP_WRITE, usage & 2 != 0);
    wgpu_usage_flags.set(BufferUsages::COPY_DST, usage & 8 != 0);
    wgpu_usage_flags.set(BufferUsages::COPY_SRC, usage & 16 != 0);
    wgpu_usage_flags.set(BufferUsages::VERTEX, usage & 32 != 0);
    wgpu_usage_flags.set(BufferUsages::INDEX, usage & 64 != 0);
    wgpu_usage_flags.set(BufferUsages::UNIFORM, usage & 128 != 0);

    let buffer = wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label.to_str().unwrap()),
        usage: wgpu_usage_flags,
        contents: data,
    });

    Box::into_raw(Box::new(buffer)) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createTexture(
    mut env: JNIEnv,
    _class: JClass,
    format_id: jint,
    width: jint,
    height: jint,
    usage: jint,
) -> jlong {
    let wm = RENDERER.get().unwrap();

    let mut wgpu_usage_flags = TextureUsages::empty();

    wgpu_usage_flags.set(TextureUsages::COPY_DST, usage & 1 != 0);
    wgpu_usage_flags.set(TextureUsages::COPY_SRC, usage & 2 != 0);
    wgpu_usage_flags.set(TextureUsages::TEXTURE_BINDING, usage & 4 != 0);
    wgpu_usage_flags.set(TextureUsages::RENDER_ATTACHMENT, usage & 8 != 0);

    let format = match format_id {
        0 => wgpu::TextureFormat::Rgba8Unorm,
        1 => wgpu::TextureFormat::R8Uint,
        2 => wgpu::TextureFormat::R8Sint,
        3 => wgpu::TextureFormat::Depth32Float,
        _ => unreachable!(),
    };

    let texture = wm.gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu_usage_flags,
        view_formats: &[],
    });

    let tav = TextureAndView {
        view: texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        }),
        texture,
        format,
    };

    Box::into_raw(Box::new(tav)) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn dropTexture(_env: JNIEnv, _class: JClass, texture: jlong) {
    unsafe {
        drop(Box::from_raw(texture as *mut TextureAndView));
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn dropBuffer(_env: JNIEnv, _class: JClass, buffer: jlong) {
    unsafe {
        drop(Box::from_raw(buffer as *mut wgpu::Buffer));
    }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn submitEncoders(_env: JNIEnv, _class: JClass, encoders: jlong, encoders_len: i32) {
    #[repr(C)]
    #[derive(Debug)]
    struct BufferWriteOrder<'a> {
        wgpu_buffer: &'a wgpu::Buffer,
        data: *mut u8,
    }

    #[repr(C)]
    #[derive(Debug)]
    struct EncoderOrder<'a> {
        render_passes: *mut McRenderPass<'a>,
        render_passes_len: u64,
        buffers: *mut BufferWriteOrder<'a>,
        buffers_len: u64,
    }

    let wm = RENDERER.get().unwrap();

    //TODO use box and allocator
    let orders = unsafe {
        Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            encoders as *mut EncoderOrder,
            encoders_len as usize,
        ))
    };

    for order in &orders {
        let buffers = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                order.buffers,
                order.buffers_len as usize,
            ))
        };

        for BufferWriteOrder { wgpu_buffer, data } in buffers {
            //SAFETY: These orders are only data from a map order, which was backed
            //by a ByteBuffer of the exact same size as the GPU buffer allocation
            wm.gpu.queue.write_buffer(wgpu_buffer, 0, unsafe {
                std::slice::from_raw_parts(data, wgpu_buffer.size() as usize)
            });
        }
    }

    let mut command_buffers = vec![];

    for order in orders {
        let mut encoder = wm
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let render_passes = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                order.render_passes,
                order.render_passes_len as usize,
            ))
        };

        for render_pass_order in render_passes {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_pass_order.target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: render_pass_order.depth_target.map(|depth| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let commands = unsafe {
                Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                    render_pass_order.commands,
                    render_pass_order.commands_len as usize,
                ))
            };

            let mut buffer_binds: HashMap<String, (&wgpu::Buffer, Range<u64>)> = HashMap::new();
            let mut texture_binds: HashMap<String, &TextureAndView> = HashMap::new();
            let mut current_pipeline = 0;

            for command in commands {
                match command {
                    RenderPassCommand::Draw { offset, count } => {
                        for (key, (buffer, range)) in &buffer_binds {
                            if current_pipeline == 0 {
                                continue;
                            }

                            match &key[..] {
                                "DynamicTransforms" => {
                                    let bind_group = wm.gpu.device.create_bind_group(
                                        &wgpu::BindGroupDescriptor {
                                            label: None,
                                            layout: wm.bind_group_layouts.get("ssbo").unwrap(),
                                            entries: &[wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::Buffer(
                                                    wgpu::BufferBinding {
                                                        buffer,
                                                        offset: range.start,
                                                        size: NonZeroU64::new(range.end),
                                                    },
                                                ),
                                            }],
                                        },
                                    );

                                    render_pass.set_bind_group(1, &bind_group, &[]);
                                }
                                "Projection" => {
                                    let bind_group = wm.gpu.device.create_bind_group(
                                        &wgpu::BindGroupDescriptor {
                                            label: None,
                                            layout: wm.bind_group_layouts.get("ssbo").unwrap(),
                                            entries: &[wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::Buffer(
                                                    wgpu::BufferBinding {
                                                        buffer,
                                                        offset: 0,
                                                        size: None,
                                                    },
                                                ),
                                            }],
                                        },
                                    );

                                    render_pass.set_bind_group(2, &bind_group, &[]);
                                }
                                _ => unimplemented!("Unimplemented shader key {key}"),
                            }
                        }

                        for (key, tav) in &texture_binds {
                            match &key[..] {
                                "Sampler0" => {
                                    let bind_group = wm.gpu.device.create_bind_group(
                                        &wgpu::BindGroupDescriptor {
                                            label: None,
                                            layout: wm
                                                .bind_group_layouts
                                                .get("texture_sampler")
                                                .unwrap(),
                                            entries: &[
                                                wgpu::BindGroupEntry {
                                                    binding: 0,
                                                    resource: wgpu::BindingResource::TextureView(
                                                        &tav.view,
                                                    ),
                                                },
                                                wgpu::BindGroupEntry {
                                                    binding: 1,
                                                    resource: wgpu::BindingResource::Sampler(
                                                        &wm.mc.texture_manager.default_sampler,
                                                    ),
                                                },
                                            ],
                                        },
                                    );

                                    render_pass.set_bind_group(0, &bind_group, &[]);
                                }
                                _ => unimplemented!("Unimplemented shader key {key}"),
                            }
                        }

                        match current_pipeline {
                            1 => render_pass
                                .set_pipeline(&BUILTIN_PIPELINES.get().unwrap().gui_textured),
                            _ => unimplemented!("Pipeline {current_pipeline} unimplemented"),
                        }

                        render_pass.draw(offset..(offset + count), 0..1);

                        buffer_binds.clear();
                        texture_binds.clear();

                        current_pipeline = 0;
                    }
                    RenderPassCommand::DrawIndexed {
                        offset,
                        count,
                        primcount,
                        i,
                    } => {
                        if current_pipeline == 0 {
                            continue;
                        }

                        for (key, (buffer, range)) in &buffer_binds {
                            match &key[..] {
                                "DynamicTransforms" => {
                                    let bind_group = wm.gpu.device.create_bind_group(
                                        &wgpu::BindGroupDescriptor {
                                            label: None,
                                            layout: wm.bind_group_layouts.get("ssbo").unwrap(),
                                            entries: &[wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::Buffer(
                                                    wgpu::BufferBinding {
                                                        buffer,
                                                        offset: range.start,
                                                        size: NonZeroU64::new(range.end),
                                                    },
                                                ),
                                            }],
                                        },
                                    );

                                    render_pass.set_bind_group(1, &bind_group, &[]);
                                }
                                "Projection" => {
                                    let bind_group = wm.gpu.device.create_bind_group(
                                        &wgpu::BindGroupDescriptor {
                                            label: None,
                                            layout: wm.bind_group_layouts.get("ssbo").unwrap(),
                                            entries: &[wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::Buffer(
                                                    wgpu::BufferBinding {
                                                        buffer,
                                                        offset: 0,
                                                        size: None,
                                                    },
                                                ),
                                            }],
                                        },
                                    );

                                    render_pass.set_bind_group(2, &bind_group, &[]);
                                }
                                _ => unimplemented!("Unimplemented shader key {key}"),
                            }
                        }

                        for (key, tav) in &texture_binds {
                            match &key[..] {
                                "Texture" => {
                                    let bind_group = wm.gpu.device.create_bind_group(
                                        &wgpu::BindGroupDescriptor {
                                            label: None,
                                            layout: wm
                                                .bind_group_layouts
                                                .get("texture_sampler")
                                                .unwrap(),
                                            entries: &[
                                                wgpu::BindGroupEntry {
                                                    binding: 0,
                                                    resource: wgpu::BindingResource::TextureView(
                                                        &tav.view,
                                                    ),
                                                },
                                                wgpu::BindGroupEntry {
                                                    binding: 1,
                                                    resource: wgpu::BindingResource::Sampler(
                                                        &wm.mc.texture_manager.default_sampler,
                                                    ),
                                                },
                                            ],
                                        },
                                    );

                                    render_pass.set_bind_group(0, &bind_group, &[]);
                                }
                                _ => unimplemented!("Unimplemented shader key {key}"),
                            }
                        }

                        match current_pipeline {
                            1 => render_pass
                                .set_pipeline(&BUILTIN_PIPELINES.get().unwrap().gui_textured),
                            _ => unimplemented!("Pipeline {current_pipeline} unimplemented"),
                        }

                        render_pass.draw_indexed(
                            count..(count + primcount),
                            offset as i32,
                            0..i as u32,
                        );

                        buffer_binds.clear();
                        texture_binds.clear();
                        current_pipeline = 0;
                    }
                    RenderPassCommand::SetIndexBuffer {
                        index_buffer,
                        index_type,
                    } => {
                        render_pass.set_index_buffer(
                            index_buffer.slice(..),
                            match index_type {
                                IndexType::I16 => wgpu::IndexFormat::Uint16,
                                IndexType::I32 => wgpu::IndexFormat::Uint32,
                            },
                        );
                    }
                    RenderPassCommand::SetVertexBuffer {
                        vertex_buffer,
                        index,
                    } => {
                        render_pass.set_vertex_buffer(index, vertex_buffer.slice(..));
                    }
                    RenderPassCommand::SetPipeline(id) => {
                        current_pipeline = id;
                    }
                    RenderPassCommand::BindTexture {
                        texture,
                        name,
                        name_len,
                    } => {
                        let name_bytes = unsafe {
                            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                                name,
                                name_len as usize,
                            ))
                        };

                        let name = String::from_utf8(name_bytes.to_vec()).unwrap();
                        texture_binds.insert(name, texture);
                    }
                    RenderPassCommand::BindBuffer {
                        buffer,
                        name,
                        name_len,
                        start,
                        end,
                    } => {
                        let name_bytes = unsafe {
                            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                                name,
                                name_len as usize,
                            ))
                        };

                        let name = String::from_utf8(name_bytes.to_vec()).unwrap();
                        buffer_binds.insert(name, (buffer, start as u64..end as u64));
                    }
                }
            }
        }

        command_buffers.push(encoder.finish());
    }

    // wm.gpu.queue.submit(command_buffers);
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn presentTexture(_env: JNIEnv, _class: JClass, texture: jlong) {
    let wm = RENDERER.get().unwrap();

    let tav = unsafe { &*(texture as *const TextureAndView) };

    //Blit the specified texture onto the surface

    let surface_texture = wm.gpu.surface.get_current_texture().unwrap();

    let texture_bg = wm.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: wm.bind_group_layouts.get("texture_sampler").unwrap(),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&tav.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&wm.mc.texture_manager.default_sampler),
            },
        ],
    });

    let mut encoder = wm
        .gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&BUILTIN_PIPELINES.get().unwrap().blit);
        render_pass.set_bind_group(0, &texture_bg, &[]);
        render_pass.draw(0..6, 0..1);
    }

    wm.gpu.queue.submit([encoder.finish()]);
    surface_texture.present();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getMaxTextureSize() -> jint {
    RENDERER
        .get()
        .unwrap()
        .gpu
        .device
        .limits()
        .max_texture_dimension_2d as jint
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getMinUniformAlignment() -> jint {
    RENDERER
        .get()
        .unwrap()
        .gpu
        .device
        .limits()
        .min_uniform_buffer_offset_alignment as jint
}
