use shaderc::ShaderKind;
use wgpu::ShaderModuleDescriptorSpirV;
use std::borrow::Cow;

pub struct Shader {
    pub frag: wgpu::ShaderModule,
    pub vert: wgpu::ShaderModule
}

#[derive(Clone, Copy)]
pub struct ShaderSource<'a> {
    pub file_name: &'a str,
    pub source: &'a str,
    pub entry_point: &'a str
}

impl Shader {
    pub fn from_glsl(
        frag: ShaderSource,
        vert: ShaderSource,
        device: &wgpu::Device,
        compiler: &mut shaderc::Compiler) -> Result<Shader, shaderc::Error> {

        let frag_v = compiler.compile_into_spirv(
            frag.source,
            ShaderKind::Fragment,
            frag.file_name,
            frag.entry_point,
            None)?;

        let vert_v = compiler.compile_into_spirv(
            vert.source,
            ShaderKind::Vertex,
            vert.file_name,
            vert.entry_point,
            None)?;

        let frag = unsafe {
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::SpirV(Cow::Borrowed(frag_v.as_binary()))
            })
        };

        let vert = unsafe {
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::SpirV(Cow::Borrowed(vert_v.as_binary()))
            })
        };

        Ok(Shader {
            frag,
            vert
        })
    }
}