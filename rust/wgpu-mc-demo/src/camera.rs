use std::f32::consts::PI;

use glam::{vec3, Mat4, Vec3};


const DEG_TO_RAD:f32 = PI/180.0;
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    #[must_use]
    pub fn new(aspect: f32) -> Self {
        Self {
            position: vec3(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            up: Vec3::Y,
            aspect,
            fovy: 90.0 * DEG_TO_RAD,
            znear: 0.001,
            zfar: 1000.0,
        }
    }

    pub fn get_direction(&self) -> Vec3 {
        vec3(
            self.yaw.cos() * (1.0 - self.pitch.sin().abs()),
            self.pitch.sin(),
            self.yaw.sin() * (1.0 - self.pitch.sin().abs()),
        )
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.get_direction(), self.up)
    }

    pub fn build_perspective_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
