extern crate nalgebra_glm;
use glm::*;
use glutin::{event::MouseButton, *};
use nalgebra_glm as glm;

use std::vec::Vec;

#[derive(Debug, Copy, Clone)]
struct MouseState {
    left: bool,
    right: bool,
    middle: bool,
}

pub struct InputSystem {
    dt: f32,
    current_down: Vec<bool>,
    prev_down: Vec<bool>,
    mouse_down: MouseState,
    mouse_prev: MouseState,
    current_mouse: Vec2,
    prev_mouse: Vec2,
    delta_mouse: Vec2,
}

impl InputSystem {
    pub fn new() -> InputSystem {
        let mut input = InputSystem {
            dt: 0.0f32,
            current_down: Vec::with_capacity(event::VirtualKeyCode::Cut as usize),
            prev_down: Vec::with_capacity(event::VirtualKeyCode::Cut as usize),
            mouse_down: MouseState {
                left: false,
                right: false,
                middle: false,
            },
            mouse_prev: MouseState {
                left: false,
                right: false,
                middle: false,
            },
            current_mouse: vec2(0.0, 0.0),
            prev_mouse: vec2(0.0, 0.0),
            delta_mouse: vec2(0.0, 0.0),
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

        self.mouse_prev = self.mouse_down;
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
            event::WindowEvent::MouseInput {
                state: st,
                button: bt,
                ..
            } => {
                if let Some(button_ref) = self.match_mouse_button(*bt) {
                    match st {
                        event::ElementState::Pressed => *button_ref = true,
                        event::ElementState::Released => *button_ref = false,
                    }
                }
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

    fn match_mouse_button(&mut self, btn: MouseButton) -> Option<&mut bool> {
        match btn {
            MouseButton::Left => Some(&mut self.mouse_down.left),
            MouseButton::Right => Some(&mut self.mouse_down.right),
            MouseButton::Middle => Some(&mut self.mouse_down.middle),
            MouseButton::Other(_) => None,
        }
    }

    pub fn is_mouse_down(&mut self, key: MouseButton) -> bool {
        if let Some(button_bool) = self.match_mouse_button(key) {
            *button_bool
        } else {
            false
        }
    }
}
