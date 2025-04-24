use std::borrow::Cow;
use std::ffi::{c_char, c_int, CString};
use std::num::NonZeroIsize;
use std::sync::Arc;
use futures::executor::block_on;
use jni::JNIEnv;
use jni::objects::{JByteBuffer, JClass, JString};
use jni::sys::{jint, jlong};
use jni_fn::jni_fn;
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowHandle};
use wgpu_mc::{wgpu, Display, WindowSize, WmRenderer};
use wgpu_mc::wgpu::{BufferUsages, CommandEncoderDescriptor, TextureUsages};
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::{MinecraftResourceManagerAdapter, RENDERER};
use crate::glfw::LWJGLGLFWWindow;

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createDevice(mut env: JNIEnv, _class: JClass, window: jlong, native_window: jlong, width: jint, height: jint) {
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

    drop(RENDERER.set(wm));
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createBuffer(mut env: JNIEnv, _class: JClass, label: JString, usage: jint, size: jint) -> jlong {
    let wm = RENDERER.get().unwrap();

    let label = env.get_string(&label).unwrap();

    let mut wgpu_usage_flags = BufferUsages::empty();
    wgpu_usage_flags.set(BufferUsages::MAP_READ, usage & 1 != 0);
    wgpu_usage_flags.set(BufferUsages::MAP_WRITE, usage & 2 != 0);
    wgpu_usage_flags.set(BufferUsages::COPY_DST, usage & 8 != 0);
    wgpu_usage_flags.set(BufferUsages::COPY_SRC, usage & 16 != 0);
    wgpu_usage_flags.set(BufferUsages::VERTEX, usage & 32 != 0);
    wgpu_usage_flags.set(BufferUsages::INDEX, usage & 64 != 0);
    wgpu_usage_flags.set(BufferUsages::UNIFORM, usage & 128 != 0);

    let buffer = wm.gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label.to_str().unwrap()),
        size: size as _,
        usage: wgpu_usage_flags,
        mapped_at_creation: false,
    });

    Box::into_raw(Box::new(buffer)) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createBufferInit(mut env: JNIEnv, _class: JClass, label: JString, usage: jint, data: JByteBuffer) -> jlong {
    let wm = RENDERER.get().unwrap();

    let label = env.get_string(&label).unwrap();
    let data = unsafe {
        std::slice::from_raw_parts(
            env.get_direct_buffer_address(&data).unwrap(),
            env.get_direct_buffer_capacity(&data).unwrap()
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
pub fn createTexture(mut env: JNIEnv, _class: JClass, format_id: jint, width: jint, height: jint, usage: jint) -> jlong {
    let wm = RENDERER.get().unwrap();

    let mut wgpu_usage_flags = TextureUsages::empty();

    wgpu_usage_flags.set(TextureUsages::COPY_DST, usage & 1 != 0);
    wgpu_usage_flags.set(TextureUsages::COPY_SRC, usage & 2 != 0);
    wgpu_usage_flags.set(TextureUsages::TEXTURE_BINDING, usage & 4 != 0);
    wgpu_usage_flags.set(TextureUsages::RENDER_ATTACHMENT, usage & 8 != 0);

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
        format: match format_id {
            0 => wgpu::TextureFormat::Rgba8Uint,
            1 => wgpu::TextureFormat::R8Uint,
            2 => wgpu::TextureFormat::R8Sint,
            3 => wgpu::TextureFormat::Depth32Float,
            _ => unreachable!()
        },
        usage: wgpu_usage_flags,
        view_formats: &[],
    });

    Box::into_raw(Box::new(texture)) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn dropTexture(_env: JNIEnv, _class: JClass, texture: jlong) {
    unsafe { drop(Box::from_raw(texture as *mut wgpu::Texture)); }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn dropBuffer(_env: JNIEnv, _class: JClass, buffer: jlong) {
    unsafe { drop(Box::from_raw(buffer as *mut wgpu::Buffer)); }
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createCommandEncoder(_env: JNIEnv, _class: JClass) -> jlong {
    let encoder = RENDERER.get().unwrap().gpu.device.create_command_encoder(&CommandEncoderDescriptor {
        label: None,
    });

    let encoder = Box::into_raw(Box::new(encoder));
    encoder as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getMaxTextureSize() -> jint {
    RENDERER.get().unwrap().gpu.device.limits().max_texture_dimension_2d as jint
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn getMinUniformAlignment() -> jint {
    RENDERER.get().unwrap().gpu.device.limits().min_uniform_buffer_offset_alignment as jint
}