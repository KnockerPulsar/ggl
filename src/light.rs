use std::format;
use std::sync::Arc;

use crate::component::Transform;
use crate::egui_drawable::EguiDrawable;
use crate::shader::ShaderProgram;

use egui::Ui;
use nalgebra_glm::*;

pub trait Light {
    fn upload_data(
        &self,
        gl: &Arc<glow::Context>, // OpenGL context
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
    pub cutoff_cosines: Vec2, // ! ANGLES IN RADIANS

    pub colors: LightColors,
    pub attenuation_constants: Vec3,
}

impl Light for DirectionalLight {
    fn upload_data(
        &self,
        gl: &Arc<glow::Context>, // OpenGL context
        transform: &Transform,

        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();

        shader.set_vec3(&gl, &format!("{}.direction", uniform_name), direction);
        shader.set_vec3(
            &gl,
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &gl,
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
        gl: &Arc<glow::Context>,
        transform: &Transform,
        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader.set_vec3(&gl, &format!("{}.position", uniform_name), position);
        shader.set_vec3(
            &gl,
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
        shader.set_vec3(
            &gl,
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
        gl: &Arc<glow::Context>, // OpenGL context
        transform: &Transform,

        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader.set_vec3(&gl, &format!("{}.position", uniform_name), position);
        shader.set_vec3(&gl, &format!("{}.direction", uniform_name), direction);
        shader.set_vec3(
            &gl,
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.attenuation_constants", uniform_name),
            self.attenuation_constants,
        );
        shader.set_vec2(
            &gl,
            &format!("{}.cutoff_cos", uniform_name),
            self.cutoff_cosines,
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl EguiDrawable for LightColors {
    fn on_egui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Diffuse: ");

            ui.label("Ambient: ");

            ui.label("Specular: ");
        });

        ui.horizontal(|ui| {
            let mut float_vec: [f32; 3] = self.diffuse.try_into().expect("KAK");
            ui.color_edit_button_rgb(&mut float_vec);
            self.diffuse.copy_from_slice(&float_vec);

            let mut float_vec: [f32; 3] = self.ambient.try_into().expect("KAK");
            ui.color_edit_button_rgb(&mut float_vec);
            self.ambient.copy_from_slice(&float_vec);

            let mut float_vec: [f32; 3] = self.specular.try_into().expect("KAK");
            ui.color_edit_button_rgb(&mut float_vec);
            self.specular.copy_from_slice(&float_vec);
        });
    }
}

impl EguiDrawable for Vec3 {
    fn on_egui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.x).speed(0.01));
            ui.add(egui::DragValue::new(&mut self.y).speed(0.01));
            ui.add(egui::DragValue::new(&mut self.z).speed(0.01));
        });
    }
}

impl EguiDrawable for Vec2 {
    fn on_egui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.x).speed(0.01));
            ui.add(egui::DragValue::new(&mut self.y).speed(0.01));
        });
    }
}

impl EguiDrawable for SpotLight {
    fn on_egui(&mut self, ui: &mut Ui) {
        ui.add(egui::Checkbox::new(&mut self.enabled, "enabled"));

        if self.enabled {
            ui.add(egui::Label::new("Cuttoff cosines"));
            self.cutoff_cosines.on_egui(ui);

            ui.add(egui::Label::new("Attenuation constants"));
            self.attenuation_constants.on_egui(ui);

            self.colors.on_egui(ui);
        }
    }
}
