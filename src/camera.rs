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

    angles: Vec2,
    movement_speed: f32,
    mouse_sensitivity: f32,
    fov: f32,
}

impl Camera {
    fn get_camera_vectors(world_up: &Vec3, euler_angles: &Vec2) -> (Vec3, Vec3, Vec3) {
        let fwd_vec = Camera::forward_from_angles(&euler_angles);
        let right_vec = cross(&fwd_vec, world_up);
        let up_vec = cross(&right_vec, &fwd_vec);

        (fwd_vec, right_vec, up_vec)
    }

    pub fn new(position: &Vec3, world_up: &Vec3, angle: &Vec2) -> Camera {
        let (fwd_vec, right_vec, up_vec) = Camera::get_camera_vectors(&world_up, &angle);

        Camera {
            pos: *position,
            forward: fwd_vec, // Calculated in update_camera_vectors()
            up: up_vec,
            right: right_vec,
            world_up: *world_up, // Hardcoded for now
            // angles(0,0,0) looks at the x axis by default
            // angles(0, -90, 0) looks at the -z axis
            angles: *angle,
            movement_speed: 2.5f32,
            mouse_sensitivity: 100f32,
            fov: 90f32,
        }
    }

    pub fn update(&mut self, input: &InputSystem) {
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

        self.angles.x -= input.mouse_delta().y * self.mouse_sensitivity * dt;
        self.angles.y -= input.mouse_delta().x * self.mouse_sensitivity * dt;

        self.angles.x = self.angles.x.clamp(-89.0, 89.0f32);

        if input.mouse_delta().norm_squared() > 0.0f32 {
            (self.forward, self.right, self.up) =
                Camera::get_camera_vectors(&self.world_up, &self.angles);
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        return look_at(&self.pos, &(self.pos + self.forward), &self.up);
    }

    fn forward_from_angles(euler_angles: &Vec2) -> Vec3 {
        let x_rot = rotation(euler_angles.x.to_radians(), &vec3(1.0, 0.0, 0.0));
        let y_rot = rotation(euler_angles.y.to_radians(), &vec3(0.0, 1.0, 0.0));

        let rot = y_rot * x_rot;

        let fwd = rot * vec4(0.0, 0.0, -1.0, 0.0);

        fwd.xyz()
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }
}
