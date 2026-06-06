use crate::device::blaze3d::{NormalizedType, RenderPipeline, UniformType};
use crate::preprocessing::shim_samplers;
use crate::{BLITTER, MinecraftResourceManagerAdapter, RENDERER, preprocessing};
use futures::executor::block_on;
use jni::JNIEnv;
use jni::objects::{JByteBuffer, JClass, JString};
use jni::sys::{jint, jlong};
use jni_fn::jni_fn;
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{
    HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use std::borrow::Cow;
use std::ffi::{CStr, CString, c_char, c_int};
use std::fs::remove_dir;
use std::io::pipe;
use std::num::NonZeroIsize;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use glsl::parser::Parse;
use glsl::syntax::ShaderStage;
use glsl::transpiler::glsl::show_translation_unit;
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt, TextureBlitter, TextureBlitterBuilder};
use wgpu_mc::wgpu::{BlendState, ShaderLocation, ShaderSource, TextureFormat, naga};
use wgpu_mc::{Display, WindowSize, WmRenderer, wgpu};

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createDevice(
    mut env: JNIEnv,
    _class: JClass,
    display: jlong,
    window: jlong,
    width: u32,
    height: u32,
) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::VALIDATION
            | wgpu::InstanceFlags::DEBUG
            | wgpu::InstanceFlags::GPU_BASED_VALIDATION,
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
        // flags: Default::default()
    });

    #[cfg(windows)]
    let surface = unsafe {
        use winapi::shared::windef::HWND;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winapi::um::winuser::{GWLP_HINSTANCE, GetWindowLongPtrW, IsWindow};
        let hwnd: HWND = window as HWND;

        println!("{hwnd:?} is window: {}", IsWindow(hwnd));

        let mut win_handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
        win_handle.hinstance =
            NonZeroIsize::new(unsafe { GetWindowLongPtrW(hwnd as _, GWLP_HINSTANCE) } as isize);

        let handle = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Windows(WindowsDisplayHandle::new())),
            raw_window_handle: RawWindowHandle::Win32(win_handle),
        };

        unsafe { instance.create_surface_unsafe(handle).unwrap() }
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
        unsafe { instance.create_surface_unsafe(handle).unwrap() }
    };

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
        max_bind_groups: adapter.limits().max_bind_groups,
        // max_storage_buffers_per_shader_stage: 50,
        ..Default::default()
    };

    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::default() | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
        // | wgpu::Features::DEPTH_CLIP_CONTROL
        // | wgpu::Features::PUSH_CONSTANTS
        // | wgpu::Features::BUFFER_BINDING_ARRAY
        // | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
        // | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        // | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
        // | wgpu::Features::MULTI_DRAW_INDIRECT,
        required_limits,
        experimental_features: Default::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: Default::default(),
    }))
    .unwrap();

    println!("Adapter: {:?}", adapter.get_info());
    println!("Formats: {:?}", surface_caps.formats);
    println!("Present modes: {:?}", surface_caps.present_modes);
    println!("Alpha modes: {:?}", surface_caps.alpha_modes);
    println!("Usages: {:?}", surface_caps.usages);
    println!("Width: {}, Height: {}", width, height);

    surface.configure(&device, &surface_config);

    println!("configured");

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

    let blitter = TextureBlitterBuilder::new(&wm.gpu.device, wgpu::TextureFormat::Bgra8Unorm)
        .sample_type(wgpu::FilterMode::Nearest)
        .build();

    drop(BLITTER.set(blitter));
    drop(RENDERER.set(wm));
}

#[unsafe(no_mangle)]
pub extern "C" fn create_command_encoder() -> Box<wgpu::CommandEncoder> {
    let wm = RENDERER.get().unwrap();

    Box::new(
        wm.gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("<wm/mc command encoder>"),
            }),
    )
}

pub struct TextureView_ {
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

#[unsafe(no_mangle)]
pub extern "C" fn create_texture_view(texture: &Texture_) -> Box<TextureView_> {
    let wm = RENDERER.get().unwrap();

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
    color_texture: &TextureView_,
    clear: bool,
    clear_color: u32,
    depth_texture: Option<&TextureView_>,
    clear_depth: bool,
    depth: f64,
) -> Box<wgpu::RenderPass<'static>> {
    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &color_texture.texture_view,
            depth_slice: None,
            resolve_target: None,
            ops: if clear {
                wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: (clear_color & 0xff) as f64 / 255.0,
                        g: ((clear_color >> 8) & 0xff) as f64 / 255.0,
                        b: ((clear_color >> 16) & 0xff) as f64 / 255.0,
                        a: ((clear_color >> 24) & 0xff) as f64 / 255.0,
                    }),
                    ..Default::default()
                }
            } else {
                wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    ..Default::default()
                }
            },
        })],
        depth_stencil_attachment: depth_texture.map(|tex| wgpu::RenderPassDepthStencilAttachment {
            view: &tex.texture_view,
            depth_ops: if clear_depth {
                Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(depth as f32),
                    ..Default::default()
                })
            } else {
                None
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
pub extern "C" fn write_mapped_buffer(buffer: &wgpu::Buffer, data: *mut u8, size: u64) {
    let wm = RENDERER.get().unwrap();
    wm.gpu.queue.write_buffer(buffer, 0, unsafe {
        std::slice::from_raw_parts(data, size as _)
    });
}

pub mod blaze3d {
    use std::ffi::c_char;
    use wgpu_mc::wgpu;

    #[repr(u64)]
    pub enum NormalizedType {
        F32x3 = 1,
        U8x8 = 2,
        U8x4 = 3,
        F32x2 = 4,
        F32 = 5,
        F32x4 = 6,
        I16x2 = 7,
        U8x4Norm = 8,
        S8x3Norm = 9,
    }

    #[repr(C)]
    pub struct VertexFormatElement {
        pub offset: u64,
        pub type_: NormalizedType,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct VertexFormat {
        pub elements: *const VertexFormatElement,
        pub elements_count: u64,
        pub vertex_size: u64,
        pub primitive: u64,
    }

    #[repr(u64)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum UniformType {
        TexelBuffer = 0,
        UBO = 1,
        Sampler = 2,
    }

    #[repr(C)]
    pub struct UniformDescriptor {
        pub type_: UniformType,
        pub name: *const c_char,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct FragState {}

    #[repr(C)]
    #[derive(Debug)]
    pub struct RenderPipeline<'a> {
        pub uniforms: *const UniformDescriptor,
        pub uniforms_count: u64,
        pub vertex_format: &'a VertexFormat,
        pub vertex_shader: *const c_char,
        pub fragment_shader: *const c_char,
        pub defines: *const [*const c_char; 2],
        pub defines_count: u64,
        pub frag_state: &'a FragState,
        pub depth: u64,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_buffer(label: *const c_char, usage: u32, size: u64) -> Box<wgpu::Buffer> {
    let wm = RENDERER.get().unwrap();

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
    buffer: &wgpu::Buffer,
    start: u64,
    length: u64,
    data: *const u8,
) {
    let wm = RENDERER.get().unwrap();

    wm.gpu
        .queue
        .write_buffer(buffer, start, std::slice::from_raw_parts(data, length as _));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn copy_buffer_to_buffer(
    encoder: &mut wgpu::CommandEncoder,
    src: &wgpu::Buffer,
    dest: &wgpu::Buffer,
    src_offset: u64,
    dest_offset: u64,
    length: u64,
) {
    let wm = RENDERER.get().unwrap();

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
pub unsafe extern "C" fn bind_texture_to_render_pass(
    render_pass: &mut wgpu::RenderPass,
    slot: u32,
    texture: &TextureView_,
) {
    render_pass.set_bind_group(slot, &texture.bind_group, &[]);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile_render_pipeline(
    render_pipeline_description: &RenderPipeline,
) -> Box<wgpu::RenderPipeline> {
    let wm = RENDERER.get().unwrap();

    let uniforms = std::slice::from_raw_parts(
        render_pipeline_description.uniforms,
        render_pipeline_description.uniforms_count as _,
    )
    .iter()
    .map(|u| {
        #[derive(Debug)]
        struct Uniform<'a> {
            name: &'a str,
            type_: UniformType,
        }

        Uniform {
            name: CStr::from_ptr(u.name).to_str().unwrap(),
            type_: u.type_,
        }
    })
    .collect::<Vec<_>>();

    let vertex_elements = std::slice::from_raw_parts(
        render_pipeline_description.vertex_format.elements,
        render_pipeline_description.vertex_format.elements_count as _,
    );

    let defines_slice = std::slice::from_raw_parts(
        render_pipeline_description.defines,
        render_pipeline_description.defines_count as _,
    );

    let defines = defines_slice
        .iter()
        .map(|[key, value]| {
            (
                CStr::from_ptr(*key).to_str().unwrap(),
                CStr::from_ptr(*value).to_str().unwrap(),
            )
        })
        .collect::<Vec<(&str, &str)>>();

    let frag_source = unsafe { CStr::from_ptr(render_pipeline_description.fragment_shader).to_str().unwrap() };
    let vert_source = unsafe { CStr::from_ptr(render_pipeline_description.vertex_shader).to_str().unwrap() };

    let mut vert_stage_ast = ShaderStage::parse(vert_source).unwrap();
    let mut frag_stage_ast = ShaderStage::parse(frag_source).unwrap();

    preprocessing::fix_version(&mut vert_stage_ast);
    preprocessing::fix_version(&mut frag_stage_ast);

    let uniform_map = uniforms.iter().enumerate().map(|(index, u)| (u.name.to_string(), index as u32)).collect();

    preprocessing::apply_layouts(&mut vert_stage_ast, &mut frag_stage_ast, uniform_map);

    println!("##vsource##\n{vert_source}");

    thread::sleep(Duration::from_millis(50));

    shim_samplers(&mut vert_stage_ast, true);

    thread::sleep(Duration::from_millis(50));

    println!("##fsource##\n{frag_source}");

    thread::sleep(Duration::from_millis(50));

    shim_samplers(&mut frag_stage_ast, false);

    thread::sleep(Duration::from_millis(50));

    let mut vert_processed = String::new();
    let mut frag_processed = String::new();

    println!("##vert##\n{vert_processed}##frag##\n{frag_processed}");

    thread::sleep(Duration::from_millis(20));

    show_translation_unit(&mut vert_processed, &vert_stage_ast);
    show_translation_unit(&mut frag_processed, &frag_stage_ast);

    let vert_module = wm
        .gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(&vert_processed),
                stage: naga::ShaderStage::Vertex,
                defines: &defines,
            },
        });

    let frag_module = wm
        .gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(&frag_processed),
                stage: naga::ShaderStage::Fragment,
                defines: &defines,
            },
        });

    let vertex_attributes = vertex_elements
        .into_iter()
        .enumerate()
        .map(|(index, e)| wgpu::VertexAttribute {
            format: match e.type_ {
                NormalizedType::F32x3 => wgpu::VertexFormat::Float32x3,
                NormalizedType::U8x8 => wgpu::VertexFormat::Uint16x4,
                NormalizedType::U8x4 => wgpu::VertexFormat::Unorm8x4,
                NormalizedType::F32x2 => wgpu::VertexFormat::Float32x2,
                NormalizedType::F32 => wgpu::VertexFormat::Float32,
                NormalizedType::F32x4 => wgpu::VertexFormat::Float32x4,
                NormalizedType::I16x2 => wgpu::VertexFormat::Sint16x2,
                NormalizedType::U8x4Norm => wgpu::VertexFormat::Unorm8x4,
                NormalizedType::S8x3Norm => wgpu::VertexFormat::Snorm8x4,
            },
            offset: e.offset,
            shader_location: index as _,
        })
        .collect::<Vec<_>>();

    // dbg!(
    //     &vertex_attributes,
    //     render_pipeline_description.vertex_format.vertex_size
    // );

    let layout = wm
        .gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &uniforms
                .iter()
                .map(|uniform| {
                    Some(match uniform.type_ {
                        UniformType::TexelBuffer => wm.bind_group_layouts.get("texture").unwrap(),
                        UniformType::UBO => wm.bind_group_layouts.get("matrix").unwrap(),
                        UniformType::Sampler => {
                            wm.bind_group_layouts.get("texture_and_sampler").unwrap()
                        }
                    })
                })
                .collect::<Vec<_>>(),
            immediate_size: 0,
        });

    let pipeline = wm
        .gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vert_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: render_pipeline_description.vertex_format.vertex_size,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attributes,
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: match render_pipeline_description.vertex_format.primitive {
                    0 | 1 => wgpu::PrimitiveTopology::LineList,
                    2 => wgpu::PrimitiveTopology::LineStrip,
                    3 => wgpu::PrimitiveTopology::PointList,
                    4 => wgpu::PrimitiveTopology::TriangleList,
                    5 => wgpu::PrimitiveTopology::TriangleStrip,
                    6 => wgpu::PrimitiveTopology::TriangleList,
                    //Quads
                    7 => wgpu::PrimitiveTopology::TriangleList,
                    _ => unimplemented!(),
                },
                strip_index_format: match render_pipeline_description.vertex_format.primitive {
                    2 | 3 => Some(wgpu::IndexFormat::Uint32),
                    _ => None,
                },
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                //TODO
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: if render_pipeline_description.depth == 1 {
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
pub extern "C" fn present_texture(
    mut encoder: Box<wgpu::CommandEncoder>,
    texture_view: &TextureView_,
) {
    let wm = RENDERER.get().unwrap();

    if let wgpu::CurrentSurfaceTexture::Success(surface_texture) =
        wm.gpu.surface.get_current_texture()
    {
        let blitter = BLITTER.get().unwrap();

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
        // wm.gpu.queue.present(surface_texture);
        // surface_texture.present();
    } else {
        panic!()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_buffer_init(
    label: *const c_char,
    usage: u32,
    data: *mut u8,
    size: u64,
) -> Box<wgpu::Buffer> {
    let wm = RENDERER.get().unwrap();

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
    format_id: u32,
    width: u32,
    height: u32,
    depth_or_layers: u32,
    usage: u32,
) -> Box<Texture_> {
    let wm = RENDERER.get().unwrap();

    let mut wgpu_usage_flags = wgpu::TextureUsages::empty();

    wgpu_usage_flags.set(wgpu::TextureUsages::COPY_DST, usage & 1 != 0);
    wgpu_usage_flags.set(wgpu::TextureUsages::COPY_SRC, usage & 2 != 0);
    wgpu_usage_flags.set(wgpu::TextureUsages::TEXTURE_BINDING, usage & 4 != 0);
    wgpu_usage_flags.set(wgpu::TextureUsages::RENDER_ATTACHMENT, usage & 8 != 0);

    let format = match format_id {
        0 => wgpu::TextureFormat::Rgba8Unorm,
        1 => wgpu::TextureFormat::R8Unorm,
        2 => wgpu::TextureFormat::R8Unorm,
        3 => wgpu::TextureFormat::Depth32Float,
        _ => unreachable!(),
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
pub extern "C" fn max_texture_size() -> u32 {
    RENDERER
        .get()
        .unwrap()
        .gpu
        .device
        .limits()
        .max_texture_dimension_2d
}

#[unsafe(no_mangle)]
pub extern "C" fn min_uniform_offset_alignment() -> u32 {
    RENDERER
        .get()
        .unwrap()
        .gpu
        .device
        .limits()
        .min_uniform_buffer_offset_alignment
}
