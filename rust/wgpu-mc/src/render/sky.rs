#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyVertex {
    pub position: [f32; 3],
}

impl SkyVertex {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SkyVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    pub fn load_vertex_light_sky() -> ([[f32; 3]; 19], [u32; 24]) {
        Self::load_vertex_sky(16.0f32)
    }

    pub fn load_vertex_dark_sky() -> ([[f32; 3]; 19], [u32; 24]) {
        Self::load_vertex_sky(-16.0f32)
    }

    //build a flat circle
    fn load_vertex_sky(level: f32) -> ([[f32; 3]; 19], [u32; 24]) {
        let offset = 512.0;
        let signum = level.signum() * offset;
        let radian_conversion = std::f32::consts::PI / 180.0f32;
        let mut vertices = [[0.0f32; 3]; 19];
        let mut indices = [0u32; 24];
        let mut count = 0;

        vertices[count] = [0.0f32, level, 0.0f32];
        count += 1;
        for i in (-180..181).step_by(45) {
            let i_f32 = i as f32;
            vertices[count] = [
                (signum * (i_f32 * radian_conversion).cos()),
                level,
                (offset * (i_f32 * radian_conversion).sin()),
            ];
            count += 1;
        }

        // sky fan to triangle list
        let mut cursor = 1u32;
        for i in (0..indices.len()).step_by(3) {
            indices[i] = 0;
            indices[i + 1] = cursor;
            cursor += 1u32;
            indices[i + 2] = cursor;
        }

        (vertices, indices)
    }

    pub fn load_fog_sphere() -> ([[f32; 3]; 35], [u32; 51]) {
        let mut vertices = [[0f32; 3]; 35];
        let mut indices = [0u32; 51];
        let mut count = 0;

        vertices[count] = [0f32, 100f32, 0f32];

        count += 1;
        let pi_x_2 = std::f32::consts::PI * 2f32;
        for o in 0..17 {
            let dx = o as f32 * pi_x_2 / 16.0;
            let dy = dx.sin();
            let dz = dx.cos();
            vertices[count] = [dy * 120.0, dz * 120.0, -dz * 40.0];
            count += 1;
        }

        // sky fan to triangle list
        let mut cursor = 1u32;
        for i in (0..indices.len()).step_by(3) {
            indices[i] = 0;
            indices[i + 1] = cursor;
            cursor += 1u32;
            indices[i + 2] = cursor;
        }

        (vertices, indices)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunMoonVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl SunMoonVertex {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SunMoonVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    // 1 2 3 1 4 3
    // 1 2 3 1 4 3
    // -1.0f32, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
    pub fn load_vertex_sun() -> [SunMoonVertex; 6] {
        [
            SunMoonVertex {
                position: [-30.0f32, 100.0, -30.0],
                tex_coords: [0.0, 0.0],
            },
            SunMoonVertex {
                position: [-30.0, 100.0, 30.0],
                tex_coords: [0.0, 1.0],
            },
            SunMoonVertex {
                position: [30.0, 100.0, 30.0],
                tex_coords: [1.0, 1.0],
            },
            SunMoonVertex {
                position: [-30.0f32, 100.0, -30.0],
                tex_coords: [0.0, 0.0],
            },
            SunMoonVertex {
                position: [30.0, 100.0, -30.0],
                tex_coords: [1.0, 0.0],
            },
            SunMoonVertex {
                position: [30.0, 100.0, 30.0],
                tex_coords: [1.0, 1.0],
            },
        ]
    }

    pub fn load_vertex_moon(moon_phase: i32) -> [SunMoonVertex; 6] {
        let top_row = moon_phase % 4;
        let bottom_row = moon_phase / 4 % 2;
        let c1r1 = top_row as f32 / 4.0;
        let c2r1 = bottom_row as f32 / 2.0;
        let c1r2 = (top_row as f32 + 1.0) / 4.0;
        let c2r2 = (bottom_row as f32 + 1.0) / 2.0;
        [
            SunMoonVertex {
                position: [-30.0f32, -100.0, 30.0],
                tex_coords: [c1r2, c2r2],
            },
            SunMoonVertex {
                position: [30.0, -100.0, 30.0],
                tex_coords: [c1r1, c2r2],
            },
            SunMoonVertex {
                position: [30.0, -100.0, -30.0],
                tex_coords: [c1r1, c2r1],
            },
            SunMoonVertex {
                position: [-30.0f32, -100.0, 30.0],
                tex_coords: [c1r2, c2r2],
            },
            SunMoonVertex {
                position: [-30.0, -100.0, -30.0],
                tex_coords: [c1r2, c2r1],
            },
            SunMoonVertex {
                position: [30.0, -100.0, -30.0],
                tex_coords: [c1r1, c2r1],
            },
        ]
    }
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct SkyboxVertex {
//     pub position: [f32; 3],
//     pub uv: [f32; 2]
// }
//
// impl SkyboxVertex {
//
//     #[must_use]
//     pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<SkyVertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &wgpu::vertex_attr_array![
//                 0 => Float32x3,
//                 1 => Float32x2
//             ]
//         }
//     }
//
// }
