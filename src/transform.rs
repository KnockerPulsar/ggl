#![allow(dead_code)]

use crate::{ecs::Ecs, egui_drawable::EguiDrawable, light::float3_slider};

use egui::Ui;
use glm::{vec3, Mat4, Vec3};
use nalgebra_glm as glm;

#[derive(Clone, Debug)]
pub struct Transform {
    pos: Vec3,
    rot: Degree3,
    scale: Vec3,

    // Stored and uploaded in column-major layout,
    model: Mat4,

    name: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Degree3(pub Vec3);
pub struct Radian3(pub Vec3);

impl Degree3 {
    pub fn xyz(x: f32, y: f32, z: f32) -> Degree3 {
        Degree3(vec3(x, y, z))
    }
}

impl From<Vec3> for Degree3 {
    fn from(value: Vec3) -> Self {
        Degree3(value)
    }
}

impl From<Degree3> for Radian3 {
    fn from(val: Degree3) -> Self {
        Radian3(vec3(
            val.0.x.to_radians(),
            val.0.y.to_radians(),
            val.0.z.to_radians(),
        ))
    }
}

impl EguiDrawable for Degree3 {
    fn on_egui(&mut self, ui: &mut Ui, _index: usize, ecs: &Ecs) -> bool {
        float3_slider(&mut self.0, ui)
    }
}

impl Transform {
    pub fn new(pos: Vec3, rot: Degree3, name: &str) -> Self {
        Self::with_scale(pos, rot, vec3(1., 1., 1.), name)
    }

    pub fn with_scale(pos: Vec3, rot: Degree3, scale: Vec3, name: &str) -> Self {
        Transform {
            pos,
            rot,
            scale,
            model: Transform::model_matrix(pos, rot, scale),
            name: String::from(name),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Transform {
        Transform {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn update_model_matrix(&mut self) -> &mut Self {
        self.model = Transform::model_matrix(self.pos, self.rot, self.scale);
        self
    }

    pub fn set_pos(&mut self, pos: Vec3) -> &mut Self {
        self.pos = pos;
        self.update_model_matrix()
    }

    pub fn translate(&mut self, translation: Vec3) -> &mut Self {
        self.pos += translation;
        self.update_model_matrix()
    }

    pub fn set_rot(&mut self, euler: Degree3) -> &mut Self {
        self.rot = euler;
        self.update_model_matrix()
    }

    pub fn set_scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale = scale;
        self.update_model_matrix()
    }

    pub fn set_name(&mut self, n: impl Into<String>) -> &mut Self {
        self.name = n.into();
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_pos(&self) -> &Vec3 {
        &self.pos
    }

    pub fn get_rot(&self) -> Degree3 {
        self.rot
    }

    pub fn get_model_matrix(&self) -> glm::Mat4 {
        self.model
    }

    pub fn model_matrix(pos: Vec3, rot: Degree3, scl: Vec3) -> glm::Mat4 {
        let translation = glm::translate(&glm::Mat4::identity(), &pos);

        let Radian3(rot) = rot.into();
        let rot_x = glm::Mat4::from_euler_angles(rot.x, 0., 0.);
        let rot_y = glm::Mat4::from_euler_angles(0., rot.y, 0.);
        let rot_z = glm::Mat4::from_euler_angles(0., 0., rot.z);
        let rotation = rot_z * rot_y * rot_x;

        translation * rotation * glm::scaling(&scl)
    }

    pub fn set_model(&mut self, mat: glm::Mat4) {
        self.model = mat;

        // https://math.stackexchange.com/questions/237369/given-this-transformation-matrix-how-do-i-decompose-it-into-translation-rotati

        let translation = mat.column_part(3, 3);
        self.pos = glm::vec3(translation[0], translation[1], translation[2]);

        self.rot = Transform::euler_from_model(&mat);
    }

    // https://stackoverflow.com/questions/15022630/how-to-calculate-the-angle-from-rotation-matrix
    fn euler_from_model(mat: &glm::Mat4) -> Degree3 {
        // let scale_x = mat.column_part(0, 3).norm();
        // let scale_y = mat.column_part(1, 3).norm();
        // let scale_z = mat.column_part(2, 3).norm();

        let rotation_submatrix = mat.fixed_slice::<3, 3>(0, 0);
        // let rot_x_col = rotation_submatrix.column(0) / scale_x;
        // let rot_y_col = rotation_submatrix.column(1) / scale_y;
        // let rot_z_col = rotation_submatrix.column(2) / scale_z;

        let mut rot = glm::Vec3::zeros();

        rot.x = rotation_submatrix[(2, 1)]
            .atan2(rotation_submatrix[(2, 2)])
            .to_degrees();

        let rot_y_atany =
            (rotation_submatrix[(2, 1)].powi(2) + rotation_submatrix[(2, 2)].powi(2)).sqrt();

        let rot_y_atanx = -rotation_submatrix[(2, 0)];

        rot.y = rot_y_atanx.atan2(rot_y_atany).to_degrees();

        rot.z = rotation_submatrix[(1, 0)]
            .atan2(rotation_submatrix[(1, 1)])
            .to_degrees();

        rot.into()
    }

    pub fn face_camera(&mut self, camera: &crate::camera::Camera) {
        let pos_diff = camera.get_pos() - self.pos;
        // self.rot.0.y = (90. - f32::atan(pos_diff.z / pos_diff.x).to_degrees()) / 2.;

        let dist = vec3(pos_diff.x, pos_diff.z, 0.).norm();
        let sign = Vec3::dot(&pos_diff, &vec3(0., 0., 1.)).signum();
        self.rot.0.x = (180. + f32::atan(sign * pos_diff.y / dist).to_degrees()) / 2.;

        self.update_model_matrix();
    }
}

impl EguiDrawable for Transform {
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool {
        let header =
            egui::CollapsingHeader::new(format!("Transform - {}", &self.name)).show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        ui.label("Translation");
                        ui.label("Rotation");
                        ui.label("Scale");
                    });

                    let layout = egui::Layout::default();
                    columns[1]
                        .with_layout(layout, |ui| {
                            ui.vertical(|ui| {
                                let any_changed = [
                                    self.pos.on_egui(ui, index, ecs),
                                    self.rot.on_egui(ui, index, ecs),
                                    self.scale.on_egui(ui, index, ecs),
                                ]
                                .iter()
                                .any(|changed| *changed);

                                any_changed
                            })
                            .inner
                        })
                        .inner
                })
            });

        let changed = header.body_returned.unwrap_or(false);
        if changed {
            self.update_model_matrix();
        }

        changed
    }
}

impl Default for Transform {
    fn default() -> Self {
        let default_pos = glm::Vec3::zeros();
        let default_rot = Degree3::default();
        let default_scale = vec3(1., 1., 1.);
        Transform {
            pos: default_pos,
            rot: default_rot,
            scale: default_scale,
            model: Transform::model_matrix(default_pos, default_rot, default_scale),
            name: String::from(""),
        }
    }
}
