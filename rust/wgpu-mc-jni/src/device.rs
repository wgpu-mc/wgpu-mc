use crate::preprocessing::{shim_samplers, RemovePointSize};
use crate::{preprocessing, MinecraftResourceManagerAdapter, BLITTER, RENDERER};
use cyntax::MacroD;
use futures::executor::block_on;
use jni::{JNIEnv, JavaVM};
use jni::objects::JClass;
use jni::sys::jlong;
use jni_fn::jni_fn;
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle,
    Win32WindowHandle, WindowsDisplayHandle,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::hash::{DefaultHasher, Hasher};
use std::io::pipe;
use std::num::{NonZero, NonZeroIsize};
use std::path::PathBuf;
use std::sync::Arc;
use glsl::parser::Parse;
use glsl::syntax::{ExternalDeclaration, Preprocessor, PreprocessorVersion, ShaderStage, TypeSpecifierNonArray};
use glsl::transpiler::glsl::show_translation_unit;
use glsl::visitor::HostMut;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt, StagingBelt, TextureBlitterBuilder};
use wgpu_mc::wgpu::{naga, BlendState, BufferAddress, CurrentSurfaceTexture, Extent3d, IndexFormat, Limits, Origin3d, PresentMode, ShaderSource, SurfaceTexture, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo};
use wgpu_mc::{wgpu, Gpu, WmRenderer};
use wgpu_mc::util::WmArena;
use crate::blaze::{GpuFormat, PrimitiveTopology, BlazeRenderPassDescriptor, RenderPipeline, UniformType};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn configure_surface(wm: &WmRenderer, width: u32, height: u32, present_mode: u32) {
    let lock = wm.gpu.surface.lock();
    let surface = lock.as_ref().unwrap();

    surface.configure(
        &wm.gpu.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width,
            height,
            present_mode: match present_mode {
                0 => wgpu::PresentMode::Fifo,
                _ => unimplemented!(),
            },
            desired_maximum_frame_latency: 0,
            alpha_mode: Default::default(),
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        }
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn drop_surface(wm: &WmRenderer) {
    wm.gpu.surface.lock().take();
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn create_device(env: JNIEnv, _: JClass) -> jlong {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12,
        flags: wgpu::InstanceFlags::VALIDATION
            | wgpu::InstanceFlags::DEBUG
            | wgpu::InstanceFlags::GPU_BASED_VALIDATION,
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
        // flags: Default::default()
    });

    let adapter = block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        }
    )).unwrap();

    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
        required_limits: Limits {
            max_bind_groups: adapter.limits().max_bind_groups,
            ..adapter.limits()
        },
        experimental_features: Default::default(),
        memory_hints: Default::default(),
        trace: Default::default(),
    })).unwrap();
    
    let gpu = Gpu {
        instance,
        adapter,
        surface: Mutex::new(None),
        device,
        queue
    };

    let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
        jvm: env.get_java_vm().unwrap(),
    });
    
    let wm = WmRenderer::new(Arc::new(gpu), resource_provider);

    let blitter = TextureBlitterBuilder::new(&wm.gpu.device, wgpu::TextureFormat::Bgra8Unorm)
        .sample_type(wgpu::FilterMode::Nearest)
        .build();

    drop(BLITTER.set(blitter));
    
    Box::into_raw(Box::new(wm)) as *mut usize as jlong
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn create_surface(wm: &WmRenderer, display: u64, window: u64) {
    #[cfg(windows)]
    let surface = {
        use winapi::shared::windef::HWND;
        use winapi::um::winuser::{GWLP_HINSTANCE, GetWindowLongPtrW};
        let hwnd: HWND = window as HWND;

        let mut win_handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
        win_handle.hinstance =
            NonZeroIsize::new(unsafe { GetWindowLongPtrW(hwnd as _, GWLP_HINSTANCE) } as isize);

        let handle = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Windows(WindowsDisplayHandle::new())),
            raw_window_handle: RawWindowHandle::Win32(win_handle),
        };

        unsafe { wm.gpu.instance.create_surface_unsafe(handle).unwrap() }
    };

    #[cfg(not(windows))]
    let surface = {
        use raw_window_handle::{XcbDisplayHandle, XlibDisplayHandle, XlibWindowHandle};

        use std::ptr::NonNull;

        let handle = XlibDisplayHandle::new(NonNull::new(display as _), 0);

        let handle = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Xlib(handle)),
            raw_window_handle: RawWindowHandle::Xlib(XlibWindowHandle::new(window as _)),
        };
        unsafe { wm.gpu.instance.create_surface_unsafe(handle).unwrap() }
    };

    let surface_arc = Arc::new(surface);

    *wm.gpu.surface.lock() = Some(surface_arc.clone());
}



//     surface.configure(&device, &surface_config);
//
//     println!("configured");
//
//     let display = Display {
//         surface,
//         device,
//         queue,
//         config: RwLock::new(surface_config),
//         instance,
//         adapter,
//     };
//
//     let resource_provider = Arc::new(MinecraftResourceManagerAdapter {
//         jvm: env.get_java_vm().unwrap(),
//     });
//
//     let wm = WmRenderer::new(display, resource_provider);
//
//     wm.init();
//
//     drop(RENDERER.set(wm));
// }

#[unsafe(no_mangle)]
pub extern "C" fn create_command_encoder(wm: &WmRenderer) -> Box<wgpu::CommandEncoder> {
    Box::new(
        wm.gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("<wm/mc command encoder>"),
            }),
    )
}

#[derive(Debug)]
pub struct TextureView_ {
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

#[unsafe(no_mangle)]
pub extern "C" fn create_texture_view(wm: &WmRenderer, texture: &Texture_) -> Box<TextureView_> {
    let texture_view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: Some(texture.format),
        dimension: Some(wgpu::TextureViewDimension::D2),
        usage: None,
        aspect: Default::default(),
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let layout_name = if texture.format == wgpu::TextureFormat::Depth32Float {
        "texture_depth"
    } else {
        "texture"
    };

    let bind_group = wm.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &wm.bind_group_layouts.get(layout_name).unwrap(),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
    });

    Box::new(TextureView_ {
        texture_view,
        bind_group,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn create_render_pass(
    encoder: &mut wgpu::CommandEncoder,
    render_pass_descriptor: &BlazeRenderPassDescriptor
) -> Box<wgpu::RenderPass<'static>> {
    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &render_pass_descriptor.attachments.iter().map(|attachment| {
            Some(
                wgpu::RenderPassColorAttachment {
                    view: &attachment.texture_view.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: match attachment.clear_value {
                        None => Default::default(),
                        Some(clear_color) => {
                            wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: clear_color[0] as f64,
                                    g: clear_color[1] as f64,
                                    b: clear_color[2] as f64,
                                    a: clear_color[3] as f64,
                                }),
                                store: Default::default(),
                            }
                        }
                    },
                }
            )
        }).collect::<Vec<_>>(),
        depth_stencil_attachment: render_pass_descriptor.depth_attachment.map(|tex| wgpu::RenderPassDepthStencilAttachment {
            view: &tex.texture_view.texture_view,
            depth_ops: match tex.clear_value {
                None => Default::default(),
                Some(clear_val) => Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(*clear_val as f32),
                    ..Default::default()
                })
            },
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    Box::new(render_pass.forget_lifetime())
}

#[unsafe(no_mangle)]
pub extern "C" fn create_buffer(wm: &WmRenderer, label: *const c_char, usage: u32, size: u64) -> Box<wgpu::Buffer> {
    let label = unsafe { CStr::from_ptr(label) };

    let mut wgpu_usage_flags = wgpu::BufferUsages::empty();
    wgpu_usage_flags.set(wgpu::BufferUsages::MAP_READ, usage & 1 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::MAP_WRITE, usage & 2 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::COPY_DST, usage & 8 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::COPY_SRC, usage & 16 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::VERTEX, usage & 32 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::INDEX, usage & 64 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::UNIFORM, usage & 128 != 0);

    let buffer = wm.gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label.to_str().unwrap()),
        size: size as _,
        usage: wgpu_usage_flags,
        mapped_at_creation: false,
    });

    Box::new(buffer)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn write_to_buffer(
    wm: &WmRenderer,
    buffer: &wgpu::Buffer,
    start: u64,
    length: u64,
    data: *const u8,
) {
    wm.
        gpu
        .queue
        .write_buffer(buffer, start, std::slice::from_raw_parts(data, length as _));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn copy_buffer_to_buffer(
    wm: &WmRenderer,
    encoder: &mut wgpu::CommandEncoder,
    src: &wgpu::Buffer,
    dest: &wgpu::Buffer,
    src_offset: u64,
    dest_offset: u64,
    length: u64,
) {
    encoder.copy_buffer_to_buffer(src, src_offset as _, dest, dest_offset, Some(length as _));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bind_render_pipeline_to_pass(
    render_pass: &mut wgpu::RenderPass,
    pipeline: &wgpu::RenderPipeline,
) {
    render_pass.set_pipeline(pipeline);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_index_buffer(pass: &mut wgpu::RenderPass, buffer: &wgpu::Buffer, int_indices: bool) {
    pass.set_index_buffer(buffer.slice(..), if int_indices { IndexFormat::Uint32 } else { IndexFormat::Uint16 });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_vertex_buffer(pass: &mut wgpu::RenderPass, slot: u32, buffer: &wgpu::Buffer, buffer_start: u64, size: u64) {
    pass.set_vertex_buffer(slot, buffer.slice(buffer_start..buffer_start + size));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn draw(pass: &mut wgpu::RenderPass, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
    pass.draw(
        first_vertex..first_vertex + vertex_count,
        first_instance..first_instance + instance_count
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn draw_indexed(pass: &mut wgpu::RenderPass, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
    pass.draw_indexed(
        first_index..first_index + index_count,
        vertex_offset,
        first_instance..first_instance + instance_count
    );
}

pub type WgpuTextureFormat = wgpu::TextureFormat;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bind_texture_to_render_pass(
    render_pass: &mut wgpu::RenderPass,
    slot: u32,
    texture: &TextureView_,
) {
    render_pass.set_bind_group(slot, &texture.bind_group, &[]);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile_render_pipeline(
    wm: &WmRenderer,
    render_pipeline_description: &RenderPipeline,
) -> Box<wgpu::RenderPipeline> {

    let frag_source = render_pipeline_description.fragment_shader.to_string();
    let vert_source = render_pipeline_description.vertex_shader.to_string();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("shader");

    if !d.exists() {
        std::fs::create_dir(&d).unwrap();
    }

    let mut h = DefaultHasher::new();
    h.write(vert_source.as_bytes());
    h.write(frag_source.as_bytes());
    let out = format!("{:x}", h.finish());

    let mut d_ = d.clone();

    d.push(format!("{out}.in"));
    std::fs::write(d, format!("## VERT ##\n{vert_source}\n\n## FRAG ##\n{frag_source}")).unwrap();

    let directives = format!("#version 440\n{}\n", render_pipeline_description.directives);

    let vert_source = format!("{directives}{vert_source}");
    let frag_source = format!("{directives}{frag_source}");

    let preprocessed_vert = cyntax::preprocess_str(&vert_source, &[]);
    let preprocessed_frag = cyntax::preprocess_str(&frag_source, &[]);

    let mut vert_stage_ast = ShaderStage::parse(preprocessed_vert).unwrap();
    let mut frag_stage_ast = ShaderStage::parse(preprocessed_frag).unwrap();

    let shimmed_uniform_offsets: HashMap<String, (u32, u32)> = render_pipeline_description.bind_group_layouts.iter().enumerate().map(|(set, blaze_bgl)| {
            blaze_bgl.entries.iter().scan(0u32, |index, entry| {
                let set = set as u32;

                Some(match entry.type_ {
                    UniformType::TexelBuffer | UniformType::UBO => {
                        *index += 1;
                        vec![
                            (entry.name.to_string(), (set, *index - 1))
                        ]
                    },
                    UniformType::Sampler => {
                        *index += 2;

                        vec![
                            (format!("{}_wm_texshim", entry.name), (set, *index - 2)),
                            (format!("{}_wm_sampler", entry.name), (set, *index - 1)),
                        ]
                    }
                })
            }).flatten().collect::<Vec<(String, (u32, u32))>>()
    }).flatten().collect();

    let mut sampler_types = HashMap::new();

    //Split the samplers, as well as do some other pre-processing
    sampler_types.extend(shim_samplers(&mut vert_stage_ast, true));
    sampler_types.extend(shim_samplers(&mut frag_stage_ast, false));

    let vertex_in_shape = render_pipeline_description.vertex_formats.iter().scan(0, |location, format| {
        Some(format.elements.iter().map(|element| {
            *location += 1;
            (element.name.to_string(), *location - 1)
        }).collect::<Vec<(String, u32)>>())
    }).flatten().collect();

    //Apply the set and binding layouts to the uniforms
    preprocessing::apply_layouts(
        &mut vert_stage_ast,
        &mut frag_stage_ast,
        shimmed_uniform_offsets,
        vertex_in_shape
    );

    vert_stage_ast.0.0.insert(0, ExternalDeclaration::Preprocessor(Preprocessor::Version(PreprocessorVersion {
        version: 440,
        profile: None,
    })));

    frag_stage_ast.0.0.insert(0, ExternalDeclaration::Preprocessor(Preprocessor::Version(PreprocessorVersion {
        version: 440,
        profile: None,
    })));

    frag_stage_ast.visit_mut(&mut RemovePointSize { is_point_var: false });
    vert_stage_ast.visit_mut(&mut RemovePointSize { is_point_var: false });

    let mut vert_processed = String::new();
    let mut frag_processed = String::new();

    show_translation_unit(&mut vert_processed, &vert_stage_ast);
    show_translation_unit(&mut frag_processed, &frag_stage_ast);

    d_.push(format!("{out}.out"));
    std::fs::write(d_, format!("## VERT ##\n{vert_processed}\n\n## FRAG ##\n{frag_processed}")).unwrap();

    let vert_module = wm.gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(&vert_processed),
                stage: naga::ShaderStage::Vertex,
                // Don't pass any defines, the shader is already preprocessed above
                defines: &[],
            },
        });

    let frag_module = wm.gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(&frag_processed),
                stage: naga::ShaderStage::Fragment,
                defines: &[],
            },
        });

    let arena = WmArena::new(64);

    let bind_group_layouts: Vec<wgpu::BindGroupLayout> = render_pipeline_description.bind_group_layouts.iter().map(|blaze_bgl| {
        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &blaze_bgl.entries.iter().scan(0, |index, entry| {
                Some(
                    match entry.type_ {
                        UniformType::TexelBuffer => {
                            *index += 1;
                            vec![
                                wgpu::BindGroupLayoutEntry {
                                    binding: *index - 1,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                }
                            ]
                        },
                        UniformType::UBO => {
                            *index += 1;
                            vec![
                                wgpu::BindGroupLayoutEntry {
                                    binding: *index - 1,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                }
                            ]
                        },
                        UniformType::Sampler => {
                            *index += 2;

                            match sampler_types.get(&*entry.name).unwrap() {
                                TypeSpecifierNonArray::SamplerCube => vec![
                                    wgpu::BindGroupLayoutEntry {
                                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                        ty: wgpu::BindingType::Texture {
                                            sample_type: Default::default(),
                                            view_dimension: wgpu::TextureViewDimension::Cube,
                                            multisampled: false,
                                        },
                                        count: None,
                                        binding: *index - 2,
                                    },
                                    wgpu::BindGroupLayoutEntry {
                                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                        count: None,
                                        binding: *index - 1,
                                    }
                                ],
                                TypeSpecifierNonArray::Sampler2D => vec![
                                    wgpu::BindGroupLayoutEntry {
                                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                        ty: wgpu::BindingType::Texture {
                                            sample_type: Default::default(),
                                            view_dimension: wgpu::TextureViewDimension::D2,
                                            multisampled: false,
                                        },
                                        count: None,
                                        binding: *index - 2,
                                    },
                                    wgpu::BindGroupLayoutEntry {
                                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                        count: None,
                                        binding: *index - 1,
                                    }
                                ],
                                _ => unimplemented!()
                            }
                        }
                    }
                )
            }).flatten().collect::<Vec<_>>(),
        };

        wm.gpu.device.create_bind_group_layout(&descriptor)
    }).collect();

    let vertex_buffers = render_pipeline_description.vertex_formats.iter().scan(0, |shader_location, vertex_format| {
        Some(wgpu::VertexBufferLayout {
            array_stride: render_pipeline_description.vertex_formats[0].vertex_size,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: arena.alloc(vertex_format.elements.iter().map(|element| {
                *shader_location += 1;

                wgpu::VertexAttribute {
                    format: match element.format {
                        GpuFormat::RGB32_FLOAT => wgpu::VertexFormat::Float32x3,
                        GpuFormat::RG16_SINT => wgpu::VertexFormat::Sint16x2,
                        GpuFormat::RG16_UINT => wgpu::VertexFormat::Uint16x2,
                        GpuFormat::RG32_SINT => wgpu::VertexFormat::Sint32x2,
                        GpuFormat::RGB32_SINT => wgpu::VertexFormat::Sint32x3,
                        GpuFormat::RG16_SNORM => wgpu::VertexFormat::Snorm16x2,
                        GpuFormat::RG32_FLOAT => wgpu::VertexFormat::Float32x2,
                        GpuFormat::RGBA8_UNORM => wgpu::VertexFormat::Unorm8x4,
                        GpuFormat::RGBA8_SNORM => wgpu::VertexFormat::Snorm8x4,
                        GpuFormat::R32_FLOAT => wgpu::VertexFormat::Float32,
                        _ => unimplemented!("Unimplemented conversion from GpuFormat {:?} to wgpu", element.format)
                    },
                    offset: element.offset,
                    shader_location: *shader_location - 1,
                }
            }).collect::<Vec<wgpu::VertexAttribute>>()),
        })
    }).collect::<Vec<wgpu::VertexBufferLayout>>();

    let layout = wm.gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().map(Option::from).collect::<Vec<_>>(),
            immediate_size: 0,
        });

    let pipeline = wm.gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vert_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &vertex_buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology: match render_pipeline_description.primitive_topology {
                    _ => wgpu::PrimitiveTopology::TriangleList,
                },
                strip_index_format: None,
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                //TODO
                polygon_mode: Default::default(),
                conservative: false,
            },
            //todo
            depth_stencil: if true {
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                })
            } else {
                None
            },
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &frag_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: Default::default(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

    Box::new(pipeline)
}

#[unsafe(no_mangle)]
pub extern "C" fn unmap_buffer(buffer: &wgpu::Buffer) {
    buffer.unmap();
}

#[unsafe(no_mangle)]
pub extern "C" fn drop_buffer_view(_: Box<wgpu::BufferView>) {

}

#[unsafe(no_mangle)]
pub extern "C" fn write_buffer_with(wm: &WmRenderer, buffer: &wgpu::Buffer, data: *const u8, len: u64) {
    let mut view = wm.gpu.queue.write_buffer_with(buffer, 0, NonZero::new(len).unwrap()).unwrap();

    view.copy_from_slice(
        unsafe {
            std::slice::from_raw_parts(data, len as usize)
        }
    );
}


#[unsafe(no_mangle)]
pub extern "C" fn allocate_gpu_buffer_mapped(wm: &WmRenderer, size: u64, usages: u64) -> Box<wgpu::Buffer> {
    Box::new(wm.gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE,
        mapped_at_creation: true,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn acquire_next_texture(
    wm: &WmRenderer
) -> *mut SurfaceTexture {
    let lock = wm.gpu.surface.lock();
    let surface = lock.as_ref().unwrap().get_current_texture();

    if let wgpu::CurrentSurfaceTexture::Success(surface_texture) =
        surface
    {
        Box::into_raw(Box::new(surface_texture))
    } else {
        0 as _
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn present_surface(
    wm: &WmRenderer,
    surface_texture: Box<SurfaceTexture>
) {
    surface_texture.present();
}

#[unsafe(no_mangle)]
pub extern "C" fn copy_buffer_to_texture(
    encoder: &mut wgpu::CommandEncoder,
    buffer: &wgpu::Buffer,
    buffer_start: u64,
    buffer_end: u64,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    destination: &Texture_,
    destination_x: u32,
    destination_y: u32,
    copy_width: u32,
    copy_height: u32,
    mip_level: u32,
    depth_or_array_layers: u32
) {
    //TODO buffer slice?

    if mip_level != 0 { return; }

    encoder.copy_buffer_to_texture(
        TexelCopyBufferInfo {
            buffer,
            layout: TexelCopyBufferLayout {
                offset: (source_width * source_y + source_x) as BufferAddress,
                bytes_per_row: Some(source_width * destination.format.block_copy_size(None).unwrap()),
                rows_per_image: Some(source_height),
            },
        },
        TexelCopyTextureInfo {
            texture: &destination.texture,
            mip_level,
            origin: Origin3d {
                x: destination_x,
                y: destination_y,
                z: 0,
            },
            aspect: Default::default(),
        },
        Extent3d {
            width: copy_width,
            height: copy_height,
            depth_or_array_layers,
        }
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn write_to_texture(
    wm: &WmRenderer,
    destination: &Texture_,
    source: *const u8,
    source_size: u64,
    mip_level: u32,
    depth_or_array_layers: u32,
    dest_x: u32,
    dest_y: u32,
    width: u32,
    height: u32
) {
    if mip_level != 0 { return; }

    wm.gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &destination.texture,
            mip_level,
            origin: Origin3d {
                x: dest_x,
                y: dest_y,
                z: 0,
            },
            aspect: Default::default(),
        },
        unsafe { std::slice::from_raw_parts(source, source_size as _) },
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(destination.format.block_copy_size(None).unwrap() * width),
            rows_per_image: Some(height),
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers,
        }
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn submit_command_encoder(
    wm: &WmRenderer,
    encoder: Box<wgpu::CommandEncoder>
) {
    wm.gpu.queue.submit([encoder.finish()]);
}

#[unsafe(no_mangle)]
pub extern "C" fn submit_render_pass(
    _: Box<wgpu::RenderPass>
) {

}

#[unsafe(no_mangle)]
pub extern "C" fn blit_from_texture(
    wm: &WmRenderer,
    texture_view: &TextureView_,
    surface_texture: &SurfaceTexture,
) {
    let blitter = BLITTER.get().unwrap();

    let mut encoder = wm.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: None,
    });

    let view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

    blitter.copy(
        &wm.gpu.device,
        &mut encoder,
        &texture_view.texture_view,
        &view,
    );

    wm.gpu.queue.submit([encoder.finish()]);
}

#[unsafe(no_mangle)]
pub extern "C" fn create_buffer_init(
    wm: &WmRenderer,
    label: *const c_char,
    usage: u32,
    data: *mut u8,
    size: u64,
) -> Box<wgpu::Buffer> {
    let label = unsafe { CStr::from_ptr(label) };
    let data = unsafe { std::slice::from_raw_parts(data, size as _) };

    let mut wgpu_usage_flags = wgpu::BufferUsages::empty();
    wgpu_usage_flags.set(wgpu::BufferUsages::MAP_READ, usage & 1 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::MAP_WRITE, usage & 2 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::COPY_DST, usage & 8 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::COPY_SRC, usage & 16 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::VERTEX, usage & 32 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::INDEX, usage & 64 != 0);
    wgpu_usage_flags.set(wgpu::BufferUsages::UNIFORM, usage & 128 != 0);

    let buffer = wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label.to_str().unwrap()),
        usage: wgpu_usage_flags,
        contents: data,
    });

    Box::new(buffer)
}

pub struct Texture_ {
    texture: Box<wgpu::Texture>,
    format: wgpu::TextureFormat,
}

#[unsafe(no_mangle)]
pub extern "C" fn create_texture(
    wm: &WmRenderer,
    format_id: GpuFormat,
    width: u32,
    height: u32,
    depth_or_layers: u32,
    usage: u32,
) -> Box<Texture_> {
    let mut wgpu_usage_flags = wgpu::TextureUsages::empty();

    wgpu_usage_flags.set(wgpu::TextureUsages::COPY_DST, usage & 1 != 0);
    wgpu_usage_flags.set(wgpu::TextureUsages::COPY_SRC, usage & 2 != 0);
    wgpu_usage_flags.set(wgpu::TextureUsages::TEXTURE_BINDING, usage & 4 != 0);
    wgpu_usage_flags.set(wgpu::TextureUsages::RENDER_ATTACHMENT, usage & 8 != 0);

    let format = match format_id {
        GpuFormat::RGBA8_UNORM => wgpu::TextureFormat::Rgba8Unorm,
        GpuFormat::R8_UNORM => wgpu::TextureFormat::R8Unorm,
        GpuFormat::D32_FLOAT => wgpu::TextureFormat::Depth32Float,
        _ => unreachable!("{format_id:?}"),
    };

    let texture = wm.gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth_or_layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu_usage_flags,
        view_formats: &[],
    });

    Box::new(Texture_ {
        texture: Box::new(texture),
        format,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn drop_texture(_: Box<Texture_>) {}

#[unsafe(no_mangle)]
pub extern "C" fn drop_texture_view(_: Box<TextureView_>) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn drop_buffer(_: Box<wgpu::Buffer>) {}

#[unsafe(no_mangle)]
pub extern "C" fn max_texture_size(wm: &WmRenderer) -> u32 {
    wm
        .gpu
        .device
        .limits()
        .max_texture_dimension_2d
}

#[unsafe(no_mangle)]
pub extern "C" fn min_uniform_offset_alignment(wm: &WmRenderer) -> u32 {
    wm
        .gpu
        .device
        .limits()
        .min_uniform_buffer_offset_alignment
}
