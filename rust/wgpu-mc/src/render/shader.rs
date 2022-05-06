use std::borrow::Cow;

use crate::mc::datapack::NamespacedResource;
use crate::mc::resource::ResourceProvider;
use crate::wgpu::{ShaderModule, ShaderModuleDescriptor};

pub trait WmShader: Send + Sync {
    fn get_frag(&self) -> (&wgpu::ShaderModule, &str);

    fn get_vert(&self) -> (&wgpu::ShaderModule, &str);
}

#[derive(Debug)]
pub struct WgslShader {
    pub shader: wgpu::ShaderModule,
    pub frag_entry: String,
    pub vert_entry: String,
}

impl WgslShader {
    pub fn init(
        resource: &NamespacedResource,
        rp: &dyn ResourceProvider,
        device: &wgpu::Device,
        frag_entry: String,
        vert_entry: String,
    ) -> Self {
        let shader_src = rp.get_resource(resource);

        let shader_src = std::str::from_utf8(&shader_src).unwrap();

        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::from(shader_src)),
        });

        Self {
            shader: module,
            frag_entry,
            vert_entry,
        }
    }
}

impl WmShader for WgslShader {
    fn get_frag(&self) -> (&ShaderModule, &str) {
        (&self.shader, &self.frag_entry)
    }

    fn get_vert(&self) -> (&ShaderModule, &str) {
        (&self.shader, &self.vert_entry)
    }
}

#[derive(Debug)]
pub struct GlslShader {
    pub frag: wgpu::ShaderModule,
    pub vert: wgpu::ShaderModule,
}

impl GlslShader {
    pub fn init(
        frag: &NamespacedResource,
        vert: &NamespacedResource,
        rp: &dyn ResourceProvider,
        device: &wgpu::Device,
    ) -> Self {
        let frag_src = rp.get_resource(frag);
        let vert_src = rp.get_resource(vert);

        let frag_src = std::str::from_utf8(&frag_src).unwrap();
        let vert_src = std::str::from_utf8(&vert_src).unwrap();

        println!("{}", frag_src);

        let frag_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::from(frag_src),
                stage: crate::naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        let vert_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::from(vert_src),
                stage: crate::naga::ShaderStage::Vertex,
                defines: Default::default(),
            },
        });

        Self {
            frag: frag_module,
            vert: vert_module,
        }
    }
}

impl WmShader for GlslShader {
    fn get_frag(&self) -> (&ShaderModule, &str) {
        (&self.frag, "main")
    }

    fn get_vert(&self) -> (&ShaderModule, &str) {
        (&self.vert, "main")
    }
}
