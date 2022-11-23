extern crate nalgebra_glm as glm;

use std::format;

use crate::egui_drawable::EguiDrawable;
use crate::shader::ShaderProgram;
use crate::Transform;

use egui::Ui;
use nalgebra_glm::*;
use tracing::enabled;

macro_rules! shared_light_fn {
    () => {
        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn set_enabled(&mut self, enabled: &bool) {
            self.enabled = *enabled;
        }
    };
}

pub trait Light {
    fn upload_data(
        &self,
        transform: &Transform,

        //* String containing uniform name into light array
        //* Example: u_point_lights[0]
        uniform_name: &str,
        shader: &ShaderProgram,
        global_enable: &bool
    );

    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: &bool);
}

pub struct LightColors {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}

impl LightColors {
    pub fn from_specular(spec: Vec3, dimming_factor: f32) -> Self {
        LightColors { 
            ambient: spec * dimming_factor * dimming_factor, 
            diffuse: spec * dimming_factor, 
            specular: spec 
        }
    }
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
        global_enable: &bool
    ) {
        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();

        shader
            .set_vec3(&format!("{}.direction", uniform_name), direction)
            .set_vec3(
                &format!("{}.ambient", uniform_name),
                self.colors.ambient,
            )
            .set_vec3(
                &format!("{}.diffuse", uniform_name),
                self.colors.diffuse,
            )
            .set_vec3(
                &format!("{}.specular", uniform_name),
                self.colors.specular,
            )
            .set_bool(&format!("{uniform_name}.is_enabled"), *global_enable && self.is_enabled());
    }

    shared_light_fn!{}
}

impl Light for PointLight {
    fn upload_data(
        &self,
        transform: &Transform,
        uniform_name: &str,
        shader: &ShaderProgram,
        global_enable: &bool
    ) {
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader
            .set_vec3(&format!("{}.position", uniform_name), position)
            .set_vec3(
                &format!("{}.ambient", uniform_name),
                self.colors.ambient,
            )
            .set_vec3(
                &format!("{}.diffuse", uniform_name),
                self.colors.diffuse,
            )
            .set_vec3(
                &format!("{}.specular", uniform_name),
                self.colors.specular,
            )
            .set_vec3(
                &format!("{}.attenuation_constants", uniform_name),
                self.attenuation_constants,
            )
            .set_bool(&format!("{uniform_name}.is_enabled"), *global_enable && self.is_enabled());
    }

    shared_light_fn!{}
}

impl Light for SpotLight {
    fn upload_data(
        &self,
        transform: &Transform,

        uniform_name: &str,
        shader: &ShaderProgram,
        global_enable: &bool
    ) {
        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader
            .set_vec3(&format!("{}.position", uniform_name), position)
            .set_vec3(&format!("{}.direction", uniform_name), direction)
            .set_vec3(
                &format!("{}.ambient", uniform_name),
                self.colors.ambient,
            )
            .set_vec3(
                &format!("{}.diffuse", uniform_name),
                self.colors.diffuse,
            )
            .set_vec3(
                &format!("{}.specular", uniform_name),
                self.colors.specular,
            )
            .set_vec3(
                &format!("{}.attenuation_constants", uniform_name),
                self.attenuation_constants,
            )
            .set_vec2(
                &format!("{}.cutoff_cos", uniform_name),
                glm::cos(&glm::radians(&self.cutoff_angles)),
            )
            .set_bool(&format!("{uniform_name}.is_enabled"), *global_enable && self.is_enabled());
    }

    shared_light_fn!{}
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
            ui.scope(|ui| {
                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = egui::Color32::LIGHT_RED;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.x).speed(0.01))
                    .changed();

                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = egui::Color32::LIGHT_GREEN;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.y).speed(0.01))
                    .changed();

                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = egui::Color32::LIGHT_BLUE;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.z).speed(0.01))
                    .changed();
                })
        });

        fields_changed
    }
}

impl EguiDrawable for Vec2 {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        let mut fields_changed = false;

        ui.horizontal(|ui| {

            ui.scope(|ui| {
                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = egui::Color32::LIGHT_RED;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.x).speed(0.01))
                    .changed();

                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = egui::Color32::LIGHT_GREEN;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.y).speed(0.01))
                    .changed();
            })
        });
        fields_changed
    }
}

macro_rules! enabled_header {
    ($self: ident, $ui: ident, $header_name: literal, $index: ident , $body: expr) => {
        let id = $ui.make_persistent_id(format!("{} {}", $header_name, $index));

        egui::collapsing_header::CollapsingState::load_with_default_open($ui.ctx(), id, false)
            .show_header($ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{} {}", $header_name, $index));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        ui.checkbox(&mut $self.enabled, "");
                    });
                });
            })
        .body( |$ui| {
            if $self.enabled {
                $body
            }
        });
    };
}

impl EguiDrawable for SpotLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {

        enabled_header!(self, ui, "Spot light", index, {
            ui.add(egui::Label::new("Cuttoff angles"));
            self.cutoff_angles.on_egui(ui, index);

            ui.add(egui::Label::new("Attenuation constants"));
            self.attenuation_constants.on_egui(ui, index);

            self.colors.on_egui(ui, index);

        });
        
        false
    }
}

impl EguiDrawable for PointLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {

        enabled_header!(self, ui, "Point light", index, {
            ui.add(egui::Label::new("Attenuation constants"));
            self.attenuation_constants.on_egui(ui, index);

            self.colors.on_egui(ui, index);
        });

        false
    }
}

impl EguiDrawable for DirectionalLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        enabled_header!(self, ui, "Directional light", index, {
            self.colors.on_egui(ui, index);
        });

        false
    }
}
