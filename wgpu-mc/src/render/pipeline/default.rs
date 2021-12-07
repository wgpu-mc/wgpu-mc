use crate::WmRenderer;
use wgpu::{SurfaceTexture, TextureView, CommandEncoder, RenderPassDescriptor, RenderPassColorAttachment, RenderPipeline, BindGroup, Buffer, RenderPass};
use crate::render::chunk::BakedChunk;
use std::rc::Rc;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::ops::Range;
use crate::render::pipeline::{WmPipeline, RenderPipelinesManager};
use crate::mc::BlockManager;
use crate::mc::chunk::{ChunkManager, Chunk};
use crate::mc::entity::Entity;
use crate::camera::Camera;

pub struct SkyPipeline {}

// impl WmPipeline for SkyPipeline {}

pub struct WorldPipeline {}

impl WmPipeline for WorldPipeline {

    fn render<'a, 'b, 'c, 'd: 'c, 'e: 'd>(&'a self, renderer: &'b WmRenderer, mut render_pass: &'c mut RenderPass<'d>, bumpalowo: &'e bumpalo::Bump) {
        let pipepines_arc = renderer.pipelines.load_full();
        let pipelines = bumpalowo.alloc(pipepines_arc);

        render_pass.set_pipeline(&pipelines.terrain_pipeline);

        let atlases = bumpalowo.alloc(renderer.mc.texture_manager.atlases.load_full());

        render_pass.set_bind_group(0, &atlases.block.material.as_ref().unwrap().bind_group, &[]);
        render_pass.set_bind_group(1, bumpalowo.alloc(renderer.mc.uniform_bind_group.load_full()), &[]);

        renderer.mc.chunks.loaded_chunks.iter().for_each(|chunk_swap| {
            let chunk = bumpalowo.alloc(chunk_swap.load_full());

            let baked_chunk = match &chunk.baked {
                None => return,
                Some(baked_chunk) => baked_chunk
            };

            baked_chunk.sections.iter().for_each(|section| {
                let parts = &[
                    &section.nonstandard,
                    &section.top,
                    &section.bottom,
                    &section.north,
                    &section.east,
                    &section.south,
                    &section.west
                ];

                //TODO: culling
                parts.iter().for_each(|&part| {
                    // println!("{}", part.vertices.len());
                    render_pass.set_vertex_buffer(0, part.buffer.slice(..));
                    render_pass.draw(0..part.vertices.len() as u32, 0..1);
                });
            });
        });
    }
}