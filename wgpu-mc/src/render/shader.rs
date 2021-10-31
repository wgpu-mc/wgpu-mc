use shaderc::ShaderKind;
use wgpu::ShaderModuleDescriptorSpirV;
use std::borrow::Cow;

pub struct Shader {
    pub frag: wgpu::ShaderModule,
    pub vert: wgpu::ShaderModule
}

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
            device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::from(frag_v.as_binary())
            })
        };

        let vert = unsafe {
            device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::from(vert_v.as_binary())
            })
        };

        Ok(Shader {
            frag,
            vert
        })
    }
}