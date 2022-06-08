extern crate nalgebra_glm;
use glm::*;
use glutin::*;
use nalgebra_glm as glm;

pub struct InputSystem {
    dt: f32,
    current_down: std::vec::Vec<bool>,
    prev_down: std::vec::Vec<bool>,
    current_mouse: Vec2,
    prev_mouse: Vec2,
    delta_mouse: Vec2,
    first_mouse: bool,
}

impl InputSystem {
    pub fn new() -> InputSystem {
        let mut input = InputSystem {
            dt: 0.0f32,
            current_down: std::vec::Vec::with_capacity(event::VirtualKeyCode::Cut as usize),
            prev_down: std::vec::Vec::with_capacity(event::VirtualKeyCode::Cut as usize),
            current_mouse: vec2(0.0, 0.0),
            prev_mouse: vec2(0.0, 0.0),
            delta_mouse: vec2(0.0, 0.0),
            first_mouse: true,
        };

        for _ in 0..event::VirtualKeyCode::Cut as usize {
            input.current_down.push(false);
            input.prev_down.push(false);
        }

        input
    }

    pub fn update(&mut self, dt: f32) {
        self.dt = dt;

        for (prev_key, current_key) in
            std::iter::zip(self.prev_down.iter_mut(), self.current_down.iter_mut())
        {
            *prev_key = *current_key;
        }
    }

    pub fn frame_end(&mut self) {
        self.delta_mouse = vec2(0.0, 0.0);
    }

    pub fn handle_events(&mut self, input_event: &glutin::event::WindowEvent) {
        match input_event {
            event::WindowEvent::KeyboardInput { input: key, .. } => match key {
                event::KeyboardInput {
                    state: key_state,
                    virtual_keycode: Some(virt_key),
                    ..
                } => match key_state {
                    event::ElementState::Pressed => self.current_down[*virt_key as usize] = true,
                    event::ElementState::Released => self.current_down[*virt_key as usize] = false,
                },
                _ => {}
            },
            event::WindowEvent::CursorMoved { position: pos, .. } => {
                self.prev_mouse = self.current_mouse;
                self.current_mouse = vec2(pos.x as f32, pos.y as f32);

                self.delta_mouse = self.current_mouse - self.prev_mouse;
            }
            _ => {}
        }
    }

    pub fn is_down(&self, key: event::VirtualKeyCode) -> bool {
        self.current_down[key as usize]
    }

    pub fn just_pressed(&self, key: event::VirtualKeyCode) -> bool {
        self.is_down(key) && !self.prev_down[key as usize]
    }

    pub fn just_released(&self, key: event::VirtualKeyCode) -> bool {
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
