extern crate nalgebra_glm as glm;

use std::format;

use crate::egui_drawable::EguiDrawable;
use crate::shader::ShaderProgram;
use crate::Transform;

use egui::Ui;
use nalgebra_glm::*;

pub trait Light {
    fn upload_data(
        self: &Self,
        transform: &Transform,

        //* String containing uniform name into light array
        //* Example: u_point_lights[0]
        uniform_name: &str,
        shader: &ShaderProgram,
    );

    fn is_enabled(&self) -> bool;
}

pub struct LightColors {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}

pub struct DirectionalLight {
    pub enabled: bool,
    pub colors: LightColors,
}

pub struct PointLight {
    pub enabled: bool,
    pub colors: LightColors,
    pub attenuation_constants: Vec3,
}

pub struct SpotLight {
    pub enabled: bool,
    pub cutoff_angles: Vec2, // Angles in degress, converted to cos(rad(angle)) on upload

    pub colors: LightColors,
    pub attenuation_constants: Vec3,
}

impl Light for DirectionalLight {
    fn upload_data(
        &self,
        transform: &Transform,

        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();

        shader.set_vec3(&format!("{}.direction", uniform_name), direction);
        shader.set_vec3(
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Light for PointLight {
    fn upload_data(
        &self,
        transform: &Transform,
        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader.set_vec3(&format!("{}.position", uniform_name), position);
        shader.set_vec3(
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
        shader.set_vec3(
            &format!("{}.attenuation_constants", uniform_name),
            self.attenuation_constants,
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Light for SpotLight {
    fn upload_data(
        &self,
        transform: &Transform,

        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader.set_vec3(&format!("{}.position", uniform_name), position);
        shader.set_vec3(&format!("{}.direction", uniform_name), direction);
        shader.set_vec3(
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
        shader.set_vec3(
            &format!("{}.attenuation_constants", uniform_name),
            self.attenuation_constants,
        );
        shader.set_vec2(
            &format!("{}.cutoff_cos", uniform_name),
            glm::cos(&glm::radians(&self.cutoff_angles)),
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl EguiDrawable for LightColors {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        egui::Grid::new("Light colors")
            .num_columns(3)
            .start_row(0)
            .show(ui, |ui| {
                ui.label("Diffuse");
                ui.label("Ambient");
                ui.label("Specular");
                ui.end_row();

                ui.color_edit_button_rgb(self.diffuse.as_mut());
                ui.color_edit_button_rgb(self.ambient.as_mut());
                ui.color_edit_button_rgb(self.specular.as_mut());
                ui.end_row();
            });

        false
    }
}

impl EguiDrawable for Vec3 {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        let mut fields_changed = false;
        ui.horizontal(|ui| {
            fields_changed |= ui
                .add(egui::DragValue::new(&mut self.x).speed(0.01))
                .changed();
            fields_changed |= ui
                .add(egui::DragValue::new(&mut self.y).speed(0.01))
                .changed();
            fields_changed |= ui
                .add(egui::DragValue::new(&mut self.z).speed(0.01))
                .changed();
        });

        fields_changed
    }
}

impl EguiDrawable for Vec2 {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        let mut fields_changed = false;

        ui.horizontal(|ui| {
            fields_changed |= ui
                .add(egui::DragValue::new(&mut self.x).speed(0.01))
                .changed();
            fields_changed |= ui
                .add(egui::DragValue::new(&mut self.y).speed(0.01))
                .changed();
        });
        fields_changed
    }
}

impl EguiDrawable for SpotLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        egui::CollapsingHeader::new(format!("Spotlight {}", index)).show(ui, |ui| {
            ui.add(egui::Checkbox::new(&mut self.enabled, "enabled"));

            if self.enabled {
                ui.add(egui::Label::new("Cuttoff angles"));
                self.cutoff_angles.on_egui(ui, index);

                ui.add(egui::Label::new("Attenuation constants"));
                self.attenuation_constants.on_egui(ui, index);

                self.colors.on_egui(ui, index);
            }
        });
        false
    }
}

impl EguiDrawable for PointLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        egui::CollapsingHeader::new(format!("Point light {}", index)).show(ui, |ui| {
            ui.add(egui::Checkbox::new(&mut self.enabled, "enabled"));

            if self.enabled {
                ui.add(egui::Label::new("Attenuation constants"));
                self.attenuation_constants.on_egui(ui, index);

                self.colors.on_egui(ui, index);
            }
        });
        false
    }
}

impl EguiDrawable for DirectionalLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        egui::CollapsingHeader::new(format!("Directional Light {}", index)).show(ui, |ui| {
            ui.add(egui::Checkbox::new(&mut self.enabled, "enabled"));

            if self.enabled {
                self.colors.on_egui(ui, index);
            }
        });

        false
    }
}
