extern crate glfw;
use glfw::{Action, Key};
extern crate nalgebra_glm;
use glm::*;
use nalgebra_glm as glm;

pub struct InputSystem {
    dt: f32,
    current_down: std::vec::Vec<bool>,
    prev_down: std::vec::Vec<bool>,
    current_mouse: Vec2,
    prev_mouse: Vec2,
    delta_mouse: Vec2,
}

impl InputSystem {
    pub fn new() -> InputSystem {
        let mut input = InputSystem {
            dt: 0.0f32,
            current_down: std::vec::Vec::with_capacity(Key::Menu as usize),
            prev_down: std::vec::Vec::with_capacity(Key::Menu as usize),
            current_mouse: vec2(0.0, 0.0),
            prev_mouse: vec2(0.0, 0.0),
            delta_mouse: vec2(0.0, 0.0),
        };

        for _ in 0..Key::Menu as usize {
            input.current_down.push(false);
            input.prev_down.push(false);
        }

        input
    }

    pub fn handle_glfw(&mut self, events: std::vec::Vec<glfw::WindowEvent>, dt: &f32) {
        self.dt = *dt;

        for (prev_key, current_key) in
            std::iter::zip(self.prev_down.iter_mut(), self.current_down.iter_mut())
        {
            *prev_key = *current_key;
        }

        self.delta_mouse = vec2(0.0, 0.0);

        for event in events {
            match event {
                glfw::WindowEvent::Key(Key::Unknown, _, _, _) => {}
                glfw::WindowEvent::Key(key, _, Action::Press, _) => {
                    self.current_down[key as usize] = true
                }
                glfw::WindowEvent::Key(key, _, Action::Release, _) => {
                    self.current_down[key as usize] = false
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    self.prev_mouse = self.current_mouse;
                    self.current_mouse = vec2(x as f32, y as f32);
                    self.delta_mouse = self.current_mouse - self.prev_mouse
                }
                _ => {}
            }
        }
    }

    pub fn is_down(&self, key: Key) -> bool {
        self.current_down[key as usize]
    }

    pub fn just_pressed(&self, key: Key) -> bool {
        self.is_down(key) && !self.prev_down[key as usize]
    }

    pub fn just_released(&self, key: Key) -> bool {
        !self.is_down(key) && self.prev_down[key as usize]
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.current_mouse
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.delta_mouse
    }

    pub fn get_dt(&self) -> f32 {
        self.dt
    }
}
