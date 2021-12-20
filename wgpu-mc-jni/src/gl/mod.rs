use std::mem::MaybeUninit;
use std::vec::Vec;

use parking_lot::RwLock;
use slab::Slab;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use pipeline::GLCommand;
use wgpu_mc::model::Material;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use arc_swap::ArcSwap;
use wgpu_mc::WmRenderer;
use wgpu::ImageDataLayout;

pub mod pipeline;

pub static mut GL_COMMANDS: MaybeUninit<Vec<GLCommand>> = MaybeUninit::uninit();
pub static mut GL_ALLOC: MaybeUninit<Slab<GlResource>> = MaybeUninit::uninit();

pub unsafe fn init() {
    GL_ALLOC = MaybeUninit::new(Slab::with_capacity(2048));
    GL_COMMANDS = MaybeUninit::new(Vec::new());
}

pub struct GlVertexAttribute {
    count: u8,
    format: GlAttributeFormat,
    attr_type: GlAttributeType,
    stride: u8
}

#[derive(Copy, Clone, Debug)]
pub enum GlAttributeFormat {
    Short,
    Int,
    Float,
    Double,
    Byte,
    UByte
}

impl GlAttributeFormat {
    pub const fn size_of(&self) -> usize {
        match self {
            GlAttributeFormat::Short => 2,
            GlAttributeFormat::Int => 4,
            GlAttributeFormat::Float => 4,
            GlAttributeFormat::Double => 8,
            GlAttributeFormat::Byte => 1,
            GlAttributeFormat::UByte => 1
        }
    }

    pub fn from_enum(int: u32) -> Self {
        match int {
            0x1400 => Self::Byte,
            0x1401 => Self::UByte,
            0x1406 => Self::Float,
            0x140A => Self::Double,
            _ => panic!("Unknown enum {}", int)
        }
    }

    pub const fn as_wgpu(&self, count: u8) -> wgpu::VertexFormat {
        match self {
            GlAttributeFormat::Short => {
                match count {
                    2 => wgpu::VertexFormat::Uint16x2,
                    4 => wgpu::VertexFormat::Uint16x4,
                    _ => panic!()
                }
            }
            GlAttributeFormat::Int => {
                match count {
                    1 => wgpu::VertexFormat::Uint32,
                    2 => wgpu::VertexFormat::Uint32x2,
                    3 => wgpu::VertexFormat::Uint32x3,
                    4 => wgpu::VertexFormat::Uint32x4,
                    _ => panic!()
                }
            }
            GlAttributeFormat::Float => {
                match count {
                    1 => wgpu::VertexFormat::Float32,
                    2 => wgpu::VertexFormat::Float32x2,
                    3 => wgpu::VertexFormat::Float32x3,
                    4 => wgpu::VertexFormat::Float32x4,
                    _ => panic!()
                }
            }
            GlAttributeFormat::Double => {
                match count {
                    1 => wgpu::VertexFormat::Float64,
                    2 => wgpu::VertexFormat::Float64x2,
                    3 => wgpu::VertexFormat::Float64x3,
                    4 => wgpu::VertexFormat::Float64x4,
                    _ => panic!()
                }
            }
            GlAttributeFormat::Byte => {
                match count {
                    2 => wgpu::VertexFormat::Unorm8x2,
                    4 => wgpu::VertexFormat::Sint32,
                    _ => panic!()
                }
            }
            GlAttributeFormat::UByte => {
                match count {
                    2 => wgpu::VertexFormat::Unorm8x2,
                    4 => wgpu::VertexFormat::Uint32,
                    _ => panic!()
                }
            }
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum GlAttributeType {
    Position,
    Normal,
    Light,
    UV,
    Color
}

// pub struct GlVertex {
//     gl_attributes: Vec<GlVertexAttribute>,
//     wgpu_attributes: Vec<wgpu::VertexAttribute>
// }
//
// impl GlVertex {
//     pub fn describe<'a, 'b: 'a>(&'b mut self) -> wgpu::VertexBufferLayout<'b> {
//         let mut offset = 0;
//         let mut shader_location = 0;
//         self.gl_attributes.iter().for_each(|pointer| {
//             let (format, offset_inc) = (
//                 pointer.format.as_wgpu(pointer.count),
//                 pointer.count as u64 * pointer.format.size_of() as u64);
//
//             self.wgpu_attributes.push(
//                 wgpu::VertexAttribute {
//                     format,
//                     offset,
//                     shader_location
//                 }
//             );
//
//             offset += offset_inc;
//             shader_location += 1;
//         });
//
//         wgpu::VertexBufferLayout {
//             array_stride: offset,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &self.wgpu_attributes[..]
//         }
//     }
// }

pub struct GlTexture {
    width: u16,
    height: u16,
    material: Option<Material>
}

pub struct GlBuffer {
    buffer: Option<Rc<wgpu::Buffer>>
}

pub enum GlResource {
    Texture(GlTexture),
    Buffer(GlBuffer)
}

pub unsafe fn gen_texture() -> usize {
    let slab = GL_ALLOC.assume_init_mut();
    slab.insert(GlResource::Texture(GlTexture {
        width: 0,
        height: 0,
        material: None
    }))
}

pub unsafe fn gen_buffer() -> usize {
    let slab = GL_ALLOC.assume_init_mut();
    slab.insert(GlResource::Buffer(GlBuffer {
        buffer: None
    }))
}

pub unsafe fn upload_buffer_data(id: usize, data: &[u8], device: &wgpu::Device) {
    let slab = GL_ALLOC.assume_init_mut();
    match slab.get_mut(id).unwrap() {
        GlResource::Texture(_) => panic!(),
        GlResource::Buffer(buf) => {
            buf.buffer = Some(Rc::new(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: data,
                usage: wgpu::BufferUsages::all()
            })));
        }
    }
}

pub unsafe fn upload_texture_data(id: usize, data: &[u8], width: u32, height: u32, renderer: &WmRenderer) {
    let slab = GL_ALLOC.assume_init_mut();
    match slab.get_mut(id).expect("Invalid texture ID") {
        GlResource::Texture(tex) => {
            // material.diffuse_texture.texture
        },
        GlResource::Buffer(_) => panic!("Invalid texture ID")
    }
}

pub unsafe fn get_texture(id: usize) -> Option<&'static Material> {
    let slab = GL_ALLOC.assume_init_mut();
    match slab.get_mut(id).expect("Invalid texture ID") {
        GlResource::Texture(tex) => {
            tex.material.as_ref()
        },
        GlResource::Buffer(_) => panic!("Invalid texture ID")
    }
}