use crate::WmRenderer;
use wgpu::{RenderPass};





use crate::render::pipeline::{WmPipeline};




use crate::util::WmArena;

pub struct SkyPipeline {}

// impl WmPipeline for SkyPipeline {}

pub struct WorldPipeline {}

impl WmPipeline for WorldPipeline {

    fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
        let pipelines_arc = renderer.pipelines.load();
        let pipelines = arena.alloc(pipelines_arc);

        render_pass.set_pipeline(&pipelines.terrain_pipeline);

        let block_atlas = arena.alloc(renderer.mc.texture_manager.block_texture_atlas.load());
        let bindable_texture = arena.alloc(block_atlas.bindable_texture.load_full());

        render_pass.set_bind_group(0, &bindable_texture.bind_group, &[]);
        render_pass.set_bind_group(1, arena.alloc(renderer.mc.camera_bind_group.load()), &[]);

        renderer.mc.chunks.loaded_chunks.iter().for_each(|chunk_swap| {
            let chunk = arena.alloc(chunk_swap.load());

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