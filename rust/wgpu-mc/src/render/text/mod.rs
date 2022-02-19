use crate::render::pipeline::WmPipeline;
use crate::WmRenderer;
use wgpu::RenderPass;
use crate::util::WmArena;
use crate::texture::UV;
use std::collections::HashMap;

pub struct TextAtlas {
    pub map: HashMap<u32, UV>
}

// pub struct TextPipeline<'a> {
//     pub atlas: &'a TextAtlas,
//     // pub text:
// }
//
// impl WmPipeline for TextPipeline {
//     fn render<'a: 'd, 'b, 'c, 'd: 'c, 'e: 'c + 'd>(&'a self, renderer: &'b WmRenderer, render_pass: &'c mut RenderPass<'d>, arena: &'c mut WmArena<'e>) {
//
//     }
// }