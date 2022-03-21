use crate::render::pipeline::WmPipeline;
use crate::WmRenderer;
use wgpu::RenderPass;
use crate::util::WmArena;
use crate::mc::entity::EntityManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use crate::model::BindableTexture;

pub struct EntityGroupInstancingFrame {
    ///The model for the entity
    pub vertex_buffer: Arc<wgpu::Buffer>,
    ///`EntityRenderInstance`s
    pub instance_buffer: Arc<wgpu::Buffer>,

    ///mat4[][] for part transforms per instance
    pub instance_transform_bind_group: Rc<wgpu::BindGroup>,
    ///vec2[] for offsets for mob variant textures
    pub texture_offsets: Arc<wgpu::BindGroup>,
    ///the texture
    pub texture: Arc<BindableTexture>,
    ///how many entities to draw
    pub instance_count: u32,
    ///how many vertices per entity
    pub vertex_count: u32
}

pub struct EntityPipeline {
    pub frames: Vec<Arc<EntityGroupInstancingFrame>>
}

impl WmPipeline for EntityPipeline {

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
        let pipelines = arena.alloc(renderer.pipelines.load_full());
        render_pass.set_pipeline(&pipelines.entity_pipeline);

        self.frames.iter().for_each(|instance_type| {
            render_pass.set_bind_group(
                0,
                arena.alloc(instance_type.instance_transform_bind_group.clone()),
                &[]
            );

            render_pass.set_bind_group(
                1,
                arena.alloc(instance_type.texture_offsets.clone()),
                &[]
            );

            render_pass.set_bind_group(
                2, &arena.alloc(instance_type.texture.clone()).bind_group,
                &[]
            );

            render_pass.set_bind_group(
                3,
                arena.alloc(renderer.mc.camera_bind_group.load_full()),
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