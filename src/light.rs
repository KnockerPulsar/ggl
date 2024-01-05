#![allow(dead_code)]
extern crate nalgebra_glm as glm;

use std::format;

use crate::ecs::Ecs;
use crate::egui_drawable::EguiDrawable;
use crate::shader::ProgramHandle;
use crate::Transform;

use egui::Ui;
use nalgebra_glm::*;

#[macro_export]
macro_rules! enabled_header {
    ($enabled: expr, $ui: ident, $header_name: literal, $index: ident, $body: expr) => {{
        let id = $ui.make_persistent_id(format!("{} {}", $header_name, $index));
        let mut changed = false;

        changed |=
            egui::collapsing_header::CollapsingState::load_with_default_open($ui.ctx(), id, true)
                .show_header($ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", $header_name, $index));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                            changed |= ui.checkbox(&mut $enabled, "").changed;
                        });
                    });
                })
                .body(|$ui| {
                    if $enabled {
                        $body
                    }
                })
                .0
                .changed;

        changed
    }};
}

pub trait LightInterface {
    fn upload_data(
        &self,
        common_data: &CommonLightData,
        transform: &Transform,

        //* String containing uniform name into light array
        //* Example: u_point_lights[0]
        uniform_name: &str,
        shader: &ProgramHandle,
        global_enable: &bool,
    );
}

#[derive(Default, Clone)]
pub struct LightColors {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}

impl LightColors {
    pub fn from_specular(spec: Vec3, dimming_factor: f32) -> Self {
        assert!(
            dimming_factor < 1.0,
            "Dimming factor > 1.0! ({dimming_factor})"
        );
        LightColors {
            ambient: spec * dimming_factor * dimming_factor,
            diffuse: spec * dimming_factor,
            specular: spec,
        }
    }

    pub fn no_ambient(spec: Vec3, dimming_factor: f32) -> Self {
        LightColors {
            ambient: Vec3::zeros(),
            diffuse: spec * dimming_factor,
            specular: spec,
        }
    }

    pub fn ambient(&mut self, amb: Vec3) -> Self {
        LightColors {
            ambient: amb,
            ..(*self)
        }
    }

    pub fn diffuse(&mut self, diff: Vec3) -> Self {
        LightColors {
            diffuse: diff,
            ..(*self)
        }
    }

    pub fn specular(&mut self, spec: Vec3) -> Self {
        LightColors {
            specular: spec,
            ..(*self)
        }
    }
}

#[derive(Clone)]
pub struct CommonLightData {
    pub enabled: bool,
    pub colors: LightColors,
}

#[derive(Default, Clone)]
pub struct DirectionalLight {}

#[derive(Clone)]
pub struct PointLight {
    pub attenuation_constants: Vec3,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            attenuation_constants: vec3(1., 1., 1.),
        }
    }
}

#[derive(Default, Clone)]
pub struct SpotLight {
    pub cutoff_angles: Vec2, // Angles in degress, converted to cos(rad(angle)) on upload
    pub attenuation_constants: Vec3,
}

impl CommonLightData {
    fn upload_data(&self, uniform_name: &str, shader: &ProgramHandle, global_enable: &bool) {
        shader
            .set_vec3(&format!("{}.ambient", uniform_name), self.colors.ambient)
            .set_vec3(&format!("{}.diffuse", uniform_name), self.colors.diffuse)
            .set_vec3(&format!("{}.specular", uniform_name), self.colors.specular)
            .set_bool(
                &format!("{uniform_name}.is_enabled"),
                *global_enable && self.enabled,
            );
    }
}

impl LightInterface for DirectionalLight {
    fn upload_data(
        &self,
        common_data: &CommonLightData,
        transform: &Transform,

        uniform_name: &str,
        shader: &ProgramHandle,
        global_enable: &bool,
    ) {
        common_data.upload_data(uniform_name, shader, global_enable);

        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();
        shader.set_vec3(&format!("{}.direction", uniform_name), direction);
    }
}

impl LightInterface for PointLight {
    fn upload_data(
        &self,
        common_data: &CommonLightData,
        transform: &Transform,

        uniform_name: &str,
        shader: &ProgramHandle,
        global_enable: &bool,
    ) {
        common_data.upload_data(uniform_name, shader, global_enable);
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader
            .set_vec3(&format!("{}.position", uniform_name), position)
            .set_vec3(
                &format!("{}.attenuation_constants", uniform_name),
                self.attenuation_constants,
            );
    }
}

impl LightInterface for SpotLight {
    fn upload_data(
        &self,
        common_data: &CommonLightData,
        transform: &Transform,

        uniform_name: &str,
        shader: &ProgramHandle,
        global_enable: &bool,
    ) {
        common_data.upload_data(uniform_name, shader, global_enable);

        let direction = (transform.get_model_matrix() * glm::vec4(0.0, -1.0, 0.0, 0.0f32)).xyz();
        let position = (transform.get_model_matrix() * glm::vec4(0.0, 0.0, 0.0, 1.0)).xyz();

        shader
            .set_vec3(&format!("{}.position", uniform_name), position)
            .set_vec3(&format!("{}.direction", uniform_name), direction)
            .set_vec3(
                &format!("{}.attenuation_constants", uniform_name),
                self.attenuation_constants,
            )
            .set_vec2(
                &format!("{}.cutoff_cos", uniform_name),
                glm::cos(&glm::radians(&self.cutoff_angles)),
            );
    }
}

impl EguiDrawable for LightColors {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
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

pub fn float3_slider(float3: &mut Vec3, ui: &mut Ui) -> bool {
    use egui::Color32;
    let colors = [
        Color32::LIGHT_RED,
        Color32::LIGHT_GREEN,
        Color32::LIGHT_BLUE,
    ];

    macro_rules! stroke_color {
        ($ui: ident, $color: expr) => {
            $ui.style_mut().visuals.widgets.inactive.fg_stroke.color = $color;
        };
    }

    let any_changed = ui.horizontal(|ui| {
        let changed = float3
            .iter_mut()
            .zip(colors)
            .map(|(float, color)| {
                stroke_color!(ui, color);
                ui.add(egui::DragValue::new(float).speed(0.01)).changed()
            })
            .collect::<Vec<bool>>();

        changed.iter().any(|b| *b)
    });

    any_changed.inner
}

impl EguiDrawable for Vec3 {
    fn on_egui(&mut self, ui: &mut Ui, _index: usize, ecs: &Ecs) -> bool {
        float3_slider(self, ui)
    }
}

impl EguiDrawable for Vec2 {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
        let mut fields_changed = false;

        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = egui::Color32::LIGHT_RED;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.x).speed(0.01))
                    .changed();

                ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
                    egui::Color32::LIGHT_GREEN;
                fields_changed |= ui
                    .add(egui::DragValue::new(&mut self.y).speed(0.01))
                    .changed();
            })
        });
        fields_changed
    }
}
impl EguiDrawable for CommonLightData {
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
        enabled_header! {self.enabled, ui, "Common Light Data", index, {
            self.colors.on_egui(ui, index, ecs);
        }}
    }
}

// TODO: Perhaps the UI for the common light data is enough?
// I think we should just draw the data specific to each light type
impl EguiDrawable for SpotLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
        let mut q = ecs.query1::<CommonLightData>().unwrap();
        let cld = q.iter_mut().nth(index).unwrap().0.as_mut().unwrap();

        enabled_header!(cld.enabled, ui, "Spot light", index, {
            ui.add(egui::Label::new("Cuttoff angles"));
            self.cutoff_angles.on_egui(ui, index, ecs);

            ui.add(egui::Label::new("Attenuation constants"));
            self.attenuation_constants.on_egui(ui, index, ecs);
        })
    }
}

impl EguiDrawable for PointLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
        let mut q = ecs.query1::<CommonLightData>().unwrap();
        let cld = q.iter_mut().nth(index).unwrap().0.as_mut().unwrap();

        enabled_header!(cld.enabled, ui, "Point light", index, {
            ui.add(egui::Label::new("Attenuation constants"));
            self.attenuation_constants.on_egui(ui, index, ecs);

            cld.on_egui(ui, index, ecs);
        });

        false
    }
}

impl EguiDrawable for DirectionalLight {
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
        let mut q = ecs.query1::<CommonLightData>().unwrap();
        let cld = q.iter_mut().nth(index).unwrap().0.as_mut().unwrap();

        enabled_header!(cld.enabled, ui, "Directional light", index, {
            cld.on_egui(ui, index, ecs);
        });

        false
    }
}
