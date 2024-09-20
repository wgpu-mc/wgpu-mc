use wgpu::{BindGroupLayout, SamplerBindingType};

use std::collections::HashMap;

pub const BLOCK_ATLAS: &str = "wgpu_mc:atlases/block";
pub const ENTITY_ATLAS: &str = "wgpu_mc:atlases/entity";

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [u16; 2],
    pub normal: [f32; 3],
    pub color: u32,
    pub uv_offset: u32,
    pub lightmap_coords: u8,
    pub ao: u8,
}

impl Vertex {
    pub const VERTEX_LENGTH: usize = 16;

    pub fn compressed(self) -> [u8; Self::VERTEX_LENGTH] {
        // XYZ: 4 bytes (1 for each axis)
        // Normal: 3 bits
        // Color: 3 bytes
        // UV: 4 bytes
        // Animated UV index: 10 bits
        // XYZ add one flag: 3 bits
        // Block light nibble: 1 byte (4 bits for block, 4 bits for sky)

        // Total: 101 bits (13 bytes)
        let mut array = [0; Self::VERTEX_LENGTH];

        let x = self.position[0] * 16.0;
        let y = self.position[1] * 16.0;
        let z = self.position[2] * 16.0;

        let x = x as u16;
        let y = y as u16;
        let z = z as u16;

        let x_byte = x as u8;
        let y_byte = y as u8;
        let z_byte = z as u8;

        let flag_byte = ((x == 256) as u8) | (((y == 256) as u8) << 1) | (((z == 256) as u8) << 2);

        //position
        array[0] = x_byte;
        array[1] = y_byte;
        array[2] = z_byte;

        //color
        array[3] = (self.color & 0xf) as u8;
        array[4] = ((self.color >> 8) & 0xf) as u8;
        array[5] = ((self.color >> 16) & 0xf) as u8;

        //U
        array[6] = self.uv[0].to_le_bytes()[0];
        array[7] = self.uv[0].to_le_bytes()[1];
        //V
        array[8] = self.uv[1].to_le_bytes()[0];
        array[9] = self.uv[1].to_le_bytes()[1];

        let normal_bits: u8 = match self.normal {
            [-1.0, 0.0, 0.0] => 0b00000100,
            [1.0, 0.0, 0.0] => 0b00000000,
            [0.0, 1.0, 0.0] => 0b00000001,
            [0.0, -1.0, 0.0] => 0b00000101,
            [0.0, 0.0, 1.0] => 0b00000010,
            [0.0, 0.0, -1.0] => 0b00000110,
            _ => unreachable!("Invalid vertex normal"),
        };

        //UV index and normal
        array[10] = self.uv_offset as u8;
        array[11] = (((self.uv_offset >> 8) as u8) & 0b11) | (normal_bits << 2) | (flag_byte << 5);
        array[12] = self.lightmap_coords;
        array[13] = self.ao;

        array
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
}

impl QuadVertex {
    const VAA: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x2,
    ];

    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VAA,
        }
    }
}

pub fn create_bind_group_layouts(device: &wgpu::Device) -> HashMap<String, BindGroupLayout> {
    [
        (
            "camera".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        ),
        (
            "texture_depth".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Depth Texture Descriptor"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }),
        ),
        (
            "texture".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout Descriptor"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            }),
        ),
        (
            "cubemap".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cubemap Bind Group Layout Descriptor"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }),
        ),
        (
            "ssbo".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        ),
        (
            "ssbo_mut".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        ),
        (
            "matrix".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Matrix Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        ),
        (
            "entity".into(),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Entity Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }, wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            }),
        ),
    ]
    .into_iter()
    .collect()
}
