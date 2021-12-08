use crate::OPENGL_TO_WGPU_MATRIX;
use cgmath::{Point3, SquareMatrix, Vector3, EuclideanSpace};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    #[must_use]
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Point3::new(-20.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            up: Vector3::unit_y(),
            aspect,
            fovy: 110.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn get_direction(&self) -> cgmath::Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * (1.0 - self.pitch.sin().abs()),
            self.pitch.sin(),
            self.yaw.sin() * (1.0 - self.pitch.sin().abs())
        )
    }

    #[must_use]
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(
            self.position,
            self.position + self.get_direction(),
            self.up
        );

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view * OPENGL_TO_WGPU_MATRIX
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
}