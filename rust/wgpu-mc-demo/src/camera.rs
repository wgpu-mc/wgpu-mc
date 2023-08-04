use cgmath::{Point3, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    #[must_use]
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            up: Vector3::unit_y(),
            aspect,
            fovy: 110.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * (1.0 - self.pitch.sin().abs()),
            self.pitch.sin(),
            self.yaw.sin() * (1.0 - self.pitch.sin().abs()),
        )
    }

    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.position, self.position + self.get_direction(), self.up)
    }

    pub fn build_rotation_matrix(&self) -> cgmath::Matrix4<f32> {
        let array: [f32; 3] = self.get_direction().into();
        cgmath::Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), array.into(), self.up)
    }

    pub fn build_perspective_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }
}
