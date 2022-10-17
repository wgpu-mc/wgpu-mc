use std::collections::HashMap;

use serde::Deserialize;

use super::pipeline::WmPipeline;

fn hash_map() -> HashMap<String, u16> {
    HashMap::new()
}

#[derive(Deserialize)]
pub struct ShaderGraphPipelines {
    #[serde(default = "hash_map")]
    pub uniforms: HashMap<String, u16>,
}

#[derive(Deserialize)]
pub struct ShaderGraphPass {
    pub shader: String,
    pub textures: Vec<String>,
    pub output: String,
}

#[derive(Deserialize)]
pub struct ShaderGraph {
    pub color_space: String,
    pub pipelines: ShaderGraphPipelines,
    pub passes: Vec<ShaderGraphPass>,
}

impl WmPipeline for ShaderGraph {
    fn name(&self) -> &'static str {
        todo!()
    }

    fn provide_shaders(
        &self,
        _wm: &crate::WmRenderer,
    ) -> HashMap<String, Box<dyn super::shader::WmShader>> {
        todo!()
    }

    fn atlases(&self) -> &'static [&'static str] {
        todo!()
    }

    fn build_wgpu_pipeline_layouts(
        &self,
        _wm: &crate::WmRenderer,
    ) -> HashMap<String, wgpu::PipelineLayout> {
        todo!()
    }

    fn build_wgpu_pipelines(
        &self,
        _wm: &crate::WmRenderer,
    ) -> HashMap<String, wgpu::RenderPipeline> {
        todo!()
    }

    fn render<
        'pipeline: 'render_pass,
        'wm,
        'pass_borrow,
        'render_pass: 'pass_borrow,
        'arena: 'pass_borrow + 'render_pass,
    >(
        &'pipeline self,

        _wm: &'wm crate::WmRenderer,
        _render_pass: &'pass_borrow mut wgpu::RenderPass<'render_pass>,
        _arena: &'pass_borrow mut crate::util::WmArena<'arena>,
    ) {
        todo!()
    }
}
