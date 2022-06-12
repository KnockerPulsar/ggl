use crate::egui_drawable::EguiDrawable;
use glm::Vec3;
use nalgebra_glm as glm;

#[derive(Clone)]
pub struct Transform {
    pos: Vec3,
    rot: Vec3,
    name: String,
    model: glm::Mat4,
}

impl Transform {
    pub fn zeros() -> Self {
        Transform {
            pos: glm::vec3(0.0, 0.0, 0.0),
            rot: glm::vec3(0.0, 0.0, 0.0),
            model: glm::Mat4::identity(),
            name: String::from(""),
        }
    }

    pub fn new(pos: Vec3, rot: Vec3, name: &str) -> Self {
        Transform {
            pos: pos,
            rot: rot,
            model: Transform::model_matrix(pos, rot),
            name: String::from(name),
        }
    }

    pub fn set_pos(&mut self, pos: Vec3) -> &mut Self {
        self.pos = pos;
        self
    }

    pub fn set_rot(&mut self, euler: Vec3) -> &mut Self {
        self.rot = euler;
        self
    }

    pub fn set_name(&mut self, n: &str) -> &mut Self {
        self.name = String::from(n);
        self
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
        egui::CollapsingHeader::new("Transform").show(ui, |ui| {
            ui.label(format!("Name: {}", &self.name));
            self.pos.on_egui(ui);
            self.rot.on_egui(ui);
        });
    }
}
