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

    fn render<'a>(&self, renderer: &'a WmRenderer, mut render_pass: RenderPass<'a>, pipelines: &'a RenderPipelinesManager, chunks: &'a [&'a Chunk], entities: &[Entity], camera: &Camera, uniform_bind_group: &BindGroup) -> wgpu::RenderPass<'a> {
        render_pass.set_pipeline(&pipelines.terrain_pipeline);

        let buffers = (0..chunks.len()).for_each(|index| {
            let baked_chunk = match &chunks[index].baked {
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
                    render_pass.set_vertex_buffer(0, part.buffer.slice(..));
                    render_pass.draw(0..part.vertices.len() as u32, 0..1);
                });
            });
        });

        render_pass
    }
}