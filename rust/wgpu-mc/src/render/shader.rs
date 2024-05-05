use std::borrow::Cow;

use crate::mc::resource::{ResourcePath, ResourceProvider};
use crate::wgpu::{ShaderModule, ShaderModuleDescriptor};

pub trait WmShader: Send + Sync {
    fn get_frag(&self) -> (&ShaderModule, &str);

    fn get_vert(&self) -> (&ShaderModule, &str);
}

#[derive(Debug)]
pub struct WgslShader {
    pub module: ShaderModule,
    pub frag_entry: String,
    pub vert_entry: String,
}

impl WgslShader {
    pub fn init(
        resource: &ResourcePath,
        rp: &dyn ResourceProvider,
        device: &wgpu::Device,
        frag_entry: String,
        vert_entry: String,
    ) -> Option<Self> {
        let shader_src = rp.get_bytes(resource)?;

        let shader_src = std::str::from_utf8(&shader_src).ok()?;

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::from(shader_src)),
        });

        Some(Self {
            module: module,
            frag_entry,
            vert_entry,
        })
    }
}

impl WmShader for WgslShader {
    fn get_frag(&self) -> (&ShaderModule, &str) {
        (&self.module, &self.frag_entry)
    }

    fn get_vert(&self) -> (&ShaderModule, &str) {
        (&self.module, &self.vert_entry)
    }
}

#[derive(Debug)]
pub struct GlslShader {
    pub frag: ShaderModule,
    pub vert: ShaderModule,
}

impl GlslShader {
    pub fn init(
        frag: &ResourcePath,
        vert: &ResourcePath,
        rp: &dyn ResourceProvider,
        device: &wgpu::Device,
    ) -> Self {
        let frag_src = rp.get_bytes(frag).unwrap();
        let vert_src = rp.get_bytes(vert).unwrap();

        let frag_src = std::str::from_utf8(&frag_src).unwrap();
        let vert_src = std::str::from_utf8(&vert_src).unwrap();

        let frag_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::from(frag_src),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        let vert_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::from(vert_src),
                stage: wgpu::naga::ShaderStage::Vertex,
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
