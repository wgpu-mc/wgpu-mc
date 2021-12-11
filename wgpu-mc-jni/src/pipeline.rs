use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::WmRenderer;
use wgpu::{RenderPass, BufferDescriptor, BufferUsages, BindGroupDescriptor, BindGroupEntry, BindGroup};
use wgpu_mc::texture::UV;
use cgmath::{Matrix2, Matrix3, Matrix4, Vector3};
use wgpu_mc::camera::UniformMatrixHelper;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use std::sync::Arc;
use dashmap::DashMap;
use arc_swap::ArcSwap;
use wgpu_mc::model::Material;

#[derive(Clone, Copy)]
pub struct GuiQuad {
    pub from: (u16, u16),
    pub dimensions: (u16, u16),
    pub texture: u32
}

pub struct GuiPipeline {
    pub quad_queue: ArcSwap<Vec<GuiQuad>>,
    pub textures: Arc<DashMap<u32, Arc<Material>>>
}

impl WmPipeline for GuiPipeline {
    fn render<'a, 'b, 'c, 'd: 'c, 'e: 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'e bumpalo::Bump) {
        let pipelines = arena.alloc(renderer.pipelines.load_full());
        let sc = renderer.surface_config.load();

        render_pass.set_pipeline(&pipelines.gui_pipeline);

        // renderer.wgpu_state.device.create_buffer();

        let ortho_matrix = cgmath::ortho(0.0, sc.width as f32, sc.height as f32, 0.0, 0.0, 1.0);

        self.quad_queue.load().iter().for_each(|quad| {
            let translation_matrix = Matrix4::from_translation(
                Vector3::new(
                    quad.from.0 as f32,
                    quad.from.1 as f32,
                    0.0
                )
            );
            let scaling_matrix = Matrix4::from_nonuniform_scale(
                quad.dimensions.0 as f32,
                quad.dimensions.1 as f32,
                1.0
            );
            let final_matrix = UniformMatrixHelper {
                view_proj: (ortho_matrix * translation_matrix * scaling_matrix).into()
            };

            let buffer = arena.alloc(renderer.wgpu_state.device.create_buffer_init(
                &BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&final_matrix),
                    usage: BufferUsages::UNIFORM
                }
            ));

            let bind_group = arena.alloc(renderer.wgpu_state.device.create_bind_group(
                &BindGroupDescriptor {
                    label: None,
                    layout: &pipelines.layouts.camera_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding()
                        }
                    ]
                }
            ));

            let texture = arena.alloc(
                (*self.textures.get(&quad.texture).unwrap()).clone()
            );

            render_pass.set_bind_group(0, &texture.bind_group, &[]);
            render_pass.set_bind_group(1, bind_group, &[]);
        });
    }
}