use crate::{OPENGL_TO_WGPU_MATRIX};
use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use cgmath::{SquareMatrix, Rad, Point3, Deg, Vector3, Angle};

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
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: 45.0,
            pitch: 0.0,
            up: Vector3::unit_y(),
            aspect,
            fovy: 90.0,
            znear: 0.1,
            zfar: 100.0
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let target = Point3::new(
            self.position.x + (self.yaw.cos()),
            self.position.y + (self.pitch.cos()),
            self.position.z + (self.yaw.sin())
        );

        let view = cgmath::Matrix4::look_at(self.position, target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub(crate) fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub(crate) fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        // self.is_up_pressed = is_pressed;
                        false
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Up => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let forward = Vector3::new(camera.yaw.cos(), camera.pitch.cos(), camera.yaw.sin());
        let forward_norm = forward.normalize();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed {
            camera.position += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward_norm * self.speed;
        }

        // Redo radius calc in case the up/ down is pressed.

        if self.is_up_pressed {
            camera.pitch -= 0.05;
        }
        if self.is_down_pressed {
            camera.pitch += 0.05;
        }

        if self.is_right_pressed {
            camera.yaw += 0.05;
        }
        if self.is_left_pressed {
            camera.yaw -= 0.05;
            // camera.position = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}