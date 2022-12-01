#![allow(dead_code)]

use crate::{egui_drawable::EguiDrawable, light::float3_slider};
use egui::Ui;
use glm::{Vec3, Mat4, vec3};
use nalgebra_glm as glm;

#[derive(Clone, Default)]
pub struct Transform {
    pos: Vec3,
    rot: Degree3,
    scale: Vec3,
    model: Mat4,
    name: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Degree3(pub f32, pub f32, pub f32);
impl Degree3 {
    pub fn to_radian(&self) -> Radian3 {
        let rad: Vec<f32> = [self.0, self.1, self.2]
            .iter_mut()
            .map(|deg| deg.to_radians())
            .collect();

        Radian3(rad[0],rad[1], rad[2])
    }

    pub fn from_deg_vec(deg_vec: &Vec3) -> Degree3 {
        Degree3(deg_vec.x, deg_vec.y, deg_vec.z)
    }

    pub fn xyz(x: f32, y: f32, z: f32) -> Degree3 {
        Degree3(x, y, z)
    }
}

impl EguiDrawable for Degree3 {
    fn on_egui(&mut self, ui: &mut Ui, _index: usize) -> bool {
        let Degree3(mut x, mut y, mut z) = self;
        let changed = float3_slider(&mut x, &mut y, &mut z, ui);

        *self = Degree3(x, y, z);
        changed
    }
}

pub struct Radian3(f32, f32, f32);

impl Transform {
    #[allow(dead_code)]
    pub fn zeros() -> Self {
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

    pub fn set_rot(&mut self, euler: Vec3) -> &mut Self {
        self.rot = Degree3::from_deg_vec(&euler);
        self.update_model_matrix()
    }

    pub fn set_name(&mut self, n: &str) -> &mut Self {
        self.name = String::from(n);
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_model_matrix(&self) -> glm::Mat4 {
        self.model
    }

    pub fn model_matrix(pos: Vec3, rot: Degree3, scl: Vec3) -> glm::Mat4 {
        let translation = glm::translate(&glm::Mat4::identity(), &pos);
        let Radian3(rot_x, rot_y, rot_z) = rot.to_radian();

        let rotation = glm::rotate_z(
            &glm::rotate_y(
                &glm::rotate_x(&translation, rot_x),
                rot_y,
            ),
            rot_z,
        );
        let scale = glm::scale(&rotation, &scl);

        scale
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
            (rotation_submatrix[(2, 1)].powi(2) 
             + rotation_submatrix[(2, 2)].powi(2)).sqrt();

        let rot_y_atanx = -rotation_submatrix[(2, 0)];

        rot.y = rot_y_atanx.atan2(rot_y_atany).to_degrees();

        rot.z = rotation_submatrix[(1, 0)]
            .atan2(rotation_submatrix[(1, 1)])
            .to_degrees();

        Degree3::from_deg_vec(&rot)
    }
}

impl EguiDrawable for Transform {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {

        let mut changed = false;

        egui::CollapsingHeader::new(format!("Transform - {}", &self.name)).show(ui, |ui| {
            let pos_changed = self.pos.on_egui(ui, index);
            let rot_changed = self.rot.on_egui(ui, index);
            let scale_changed = self.scale.on_egui(ui, index);

            changed = pos_changed || rot_changed || scale_changed;

            if changed {
                self.update_model_matrix();
            }
        });

        changed
    }
}
