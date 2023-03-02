extern crate nalgebra_glm;

use crate::InputSystem;
use glm::*;
use glutin::event;
use nalgebra_glm as glm;

pub struct Camera {
    pos: Vec3,

    forward: Vec3,
    up: Vec3,
    right: Vec3,

    world_up: Vec3,

    // Rotation about the x and y axis
    // no z axis (tilt)
    orientation: Vec2,

    movement_speed: f32,
    mouse_sensitivity: f32,

    fov: f32,
    aspect_ratio: f32
}

impl Camera {
    fn get_camera_vectors(world_up: Vec3, euler_angles: Vec2) -> (Vec3, Vec3, Vec3) {
        let fwd_vec = Camera::forward_from_angles(euler_angles);
        let right_vec = cross(&fwd_vec, &world_up);
        let up_vec = cross(&right_vec, &fwd_vec);

        (fwd_vec, right_vec, up_vec)
    }

    pub fn new(
        position: Vec3, 
        world_up: Vec3, 
        angle: Vec2,
        aspect_ratio: f32
    ) -> Camera {
        let (fwd_vec, right_vec, up_vec) = Camera::get_camera_vectors(world_up, angle);

        Camera {
            pos: position,

            forward: fwd_vec, // Calculated in update_camera_vectors()
            up: up_vec,
            right: right_vec,

            world_up, // Hardcoded for now
                                 
            // angles(0,0,0) looks at the x axis by default
            // angles(0, -90, 0) looks at the -z axis
            orientation: angle,
            movement_speed: 10f32,
            mouse_sensitivity: 100f32,

            fov: 90f32,
            aspect_ratio
        }
    }

    pub fn update(&mut self, input: &mut InputSystem) {
        let dt = input.get_dt();

        if input.is_down(event::VirtualKeyCode::W) {
            self.pos += dt * self.forward * self.movement_speed;
        }

        if input.is_down(event::VirtualKeyCode::S) {
            self.pos -= dt * self.forward * self.movement_speed;
        }

        if input.is_down(event::VirtualKeyCode::A) {
            self.pos -= dt * self.right * self.movement_speed;
        }

        if input.is_down(event::VirtualKeyCode::D) {
            self.pos += dt * self.right * self.movement_speed;
        }

        if input.is_down(event::VirtualKeyCode::Q) {
            self.pos -= dt * self.up * self.movement_speed;
        }

        if input.is_down(event::VirtualKeyCode::E) {
            self.pos += dt * self.up * self.movement_speed;
        }

        if input.is_mouse_down(event::MouseButton::Right) {
            self.orientation.x -= input.mouse_delta().y * self.mouse_sensitivity * dt;
            self.orientation.y -= input.mouse_delta().x * self.mouse_sensitivity * dt;

            self.orientation.x = self.orientation.x.clamp(-89.0, 89.0f32);

            if input.mouse_delta().norm_squared() > 0.0f32 {
                (self.forward, self.right, self.up) =
                    Camera::get_camera_vectors(self.world_up, self.orientation);
            }
        }

    }

    pub fn get_proj_matrix(&self) -> glm::Mat4 {
        glm::perspective(
            self.aspect_ratio,
            self.get_fov_euler().to_radians(),
            0.01,
            1000.0,
        )
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        look_at(&self.pos, &(self.pos + self.forward), &self.up)
    }

    fn forward_from_angles(euler_angles: Vec2) -> Vec3 {
        let x_rot = rotation(euler_angles.x.to_radians(), &vec3(1.0, 0.0, 0.0));
        let y_rot = rotation(euler_angles.y.to_radians(), &vec3(0.0, 1.0, 0.0));

        let rot = y_rot * x_rot;

        let fwd = rot * vec4(0.0, 0.0, -1.0, 0.0);

        fwd.xyz()
    }

    pub fn get_fov_euler(&self) -> f32 {
        self.fov
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn update_aspect_ratio(&mut self, window_width: f32, window_height: f32) {
        self.aspect_ratio = window_width / window_height;
    }
}
