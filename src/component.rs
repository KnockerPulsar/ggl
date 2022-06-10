use nalgebra_glm as glm;
use nalgebra_glm::Vec3;

use crate::egui_drawable::EguiDrawable;
use crate::light::*;

pub struct Transform {
    pos: Vec3,
    rot: Vec3,
    model: glm::Mat4,
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            pos: glm::vec3(0.0, 0.0, 0.0),
            rot: glm::vec3(0.0, 0.0, 0.0),
            model: glm::Mat4::identity(),
        }
    }

    pub fn from_pos_rot(pos: Vec3, rot: Vec3) -> Self {
        Transform {
            pos: pos,
            rot: rot,
            model: glm::Mat4::identity(),
        }
    }

    pub fn get_model_matrix(&self) -> glm::Mat4 {
        return Transform::model_matrix(self.pos, self.rot);
    }

    pub fn model_matrix(pos: Vec3, rot: Vec3) -> glm::Mat4 {
        glm::rotate_z(
            &glm::rotate_y(
                &glm::rotate_x(&glm::translate(&glm::Mat4::identity(), &pos), rot.x),
                rot.y,
            ),
            rot.z,
        )
    }

    pub fn set_model(&mut self, mat: glm::Mat4) {
        self.model = mat;
    }
}


impl EguiDrawable for Transform {
    fn on_egui(&mut self, ui: &mut egui::Ui) {
        self.pos.on_egui(ui);
        self.rot.on_egui(ui);
    }
}