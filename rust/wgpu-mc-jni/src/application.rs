use std::sync::Arc;

use futures::executor::block_on;
use jni::{objects::JValue, JavaVM};
use once_cell::sync::OnceCell;
use parking_lot::lock_api::{Mutex, RwLock};
use wgpu_mc::{
    render::graph::Geometry,
    wgpu::{
        self,
        util::{BufferInitDescriptor, DeviceExt},
        BufferAddress, BufferBindingType, PresentMode,
    },
    Display, WmRenderer,
};

use crate::{
    gl::ElectrumVertex,
    RENDER_GRAPH,
};
use std::collections::HashMap;
use wgpu_mc::render::{
    graph::{RenderGraph, ResourceBacking},
    shaderpack::ShaderPackConfig,
};

pub static SHOULD_STOP: OnceCell<()> = OnceCell::new();

pub fn load_shaders(wm: &WmRenderer) {
    let shader_pack: ShaderPackConfig =
        serde_yaml::from_str(include_str!("../graph.yaml")).unwrap();

    let mut render_resources = HashMap::new();

    let mat4_projection = create_matrix_buffer(wm);
    let mat4_view = create_matrix_buffer(wm);
    let mat4_model = create_matrix_buffer(wm);

    render_resources.insert(
        "@mat4_view".into(),
        ResourceBacking::Buffer(mat4_view.clone(), BufferBindingType::Uniform),
    );

    render_resources.insert(
        "@mat4_perspective".into(),
        ResourceBacking::Buffer(mat4_projection.clone(), BufferBindingType::Uniform),
    );

    render_resources.insert(
        "@mat4_model".into(),
        ResourceBacking::Buffer(mat4_model.clone(), BufferBindingType::Uniform),
    );

    let mut custom_bind_groups = HashMap::new();
    custom_bind_groups.insert(
        "@texture_electrum_gui".into(),
        wm.bind_group_layouts.get("texture").unwrap(),
    );
    custom_bind_groups.insert(
        "@mat4_electrum_gui".into(),
        wm.bind_group_layouts.get("matrix").unwrap(),
    );

    let mut custom_geometry = HashMap::new();
    custom_geometry.insert(
        "@geo_electrum_gui".into(),
        vec![wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ElectrumVertex>() as BufferAddress,
            step_mode: Default::default(),
            attributes: &ElectrumVertex::VAO,
        }],
    );

    let render_graph = RenderGraph::new(
        wm,
        shader_pack,
        render_resources,
        Some(custom_bind_groups),
        Some(custom_geometry),
    );

    match RENDER_GRAPH.get() {
        None => {
            RENDER_GRAPH.set(Mutex::new(render_graph)).unwrap();
        }
        Some(mutex) => {
            *mutex.lock() = render_graph;
        }
    }
}

fn create_matrix_buffer(wm: &WmRenderer) -> Arc<wgpu::Buffer> {
    Arc::new(wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: &[0; 64],
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    }))
}
