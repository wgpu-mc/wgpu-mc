use crate::render::pipeline::WmPipeline;
use crate::WmRenderer;
use wgpu::RenderPass;
use crate::util::WmArena;
use crate::mc::entity::EntityManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct EntityTypeInstanceFrame {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub instance_buffer: Arc<wgpu::Buffer>,

    pub instance_transform_bind_group: Rc<wgpu::BindGroup>,
    pub texture_bind_group: Arc<wgpu::BindGroup>,
    pub instance_count: u32,
    pub vertex_count: u32
}

pub struct EntityPipeline {
    pub frame: Vec<EntityTypeInstanceFrame>
}

impl WmPipeline for EntityPipeline {

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
        let pipelines = arena.alloc(renderer.pipelines.load_full());
        render_pass.set_pipeline(&pipelines.entity_pipeline);

        render_pass.set_bind_group(
            3,
            arena.alloc(renderer.mc.camera_bind_group.load_full()),
            &[]
        );

        self.frame.iter().for_each(|instance_type| {
            render_pass.set_bind_group(
                0,
                arena.alloc(instance_type.instance_transform_bind_group.clone()),
                &[]
            );

            render_pass.set_bind_group(
                1,
                arena.alloc(instance_type.texture_bind_group.clone()),
                &[]
            );

            render_pass.set_vertex_buffer(
                0,
                arena.alloc(instance_type.vertex_buffer.clone()).slice(..)
            );

            render_pass.set_vertex_buffer(
                1,
                arena.alloc(instance_type.instance_buffer.clone()).slice(..)
            );

            render_pass.draw(0..instance_type.vertex_count, 0..instance_type.instance_count);
        });
    }

}