use crate::egui_drawable::EguiDrawable;
use egui::Ui;
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
    #[allow(dead_code)]
    pub fn zeros() -> Self {
        Transform {
            pos: glm::vec3(0.0, 0.0, 0.0),
            rot: glm::vec3(0.0, 0.0, 0.0),
            model: Transform::model_matrix(glm::Vec3::zeros(), glm::Vec3::zeros()),
            name: String::from(""),
        }
    }

    pub fn new(pos: Vec3, rot: Vec3, name: &str) -> Self {
        Transform {
            pos: pos,
            rot: rot,
            model: Transform::model_matrix(pos, glm::radians(&rot)),
            name: String::from(name),
        }
    }

    #[allow(dead_code)]
    pub fn set_pos(&mut self, pos: Vec3) -> &mut Self {
        self.pos = pos;
        self.model = Transform::model_matrix(self.pos, self.rot);
        self
    }
    #[allow(dead_code)]
    pub fn set_rot(&mut self, euler: Vec3) -> &mut Self {
        self.rot = euler;
        self.model = Transform::model_matrix(self.pos, self.rot);
        self
    }

    #[allow(dead_code)]
    pub fn set_name(&mut self, n: &str) -> &mut Self {
        self.name = String::from(n);
        self
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    pub fn get_model_matrix(&self) -> glm::Mat4 {
        self.model
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

        // https://math.stackexchange.com/questions/237369/given-this-transformation-matrix-how-do-i-decompose-it-into-translation-rotati

        let translation = mat.column_part(3, 3);
        self.pos = glm::vec3(translation[0], translation[1], translation[2]);

        self.rot = Transform::euler_from_model(&mat);
    }

    // https://stackoverflow.com/questions/15022630/how-to-calculate-the-angle-from-rotation-matrix
    fn euler_from_model(mat: &glm::Mat4) -> glm::Vec3 {
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

        rot
    }
}

impl EguiDrawable for Transform {
    fn on_egui(&mut self, ui: &mut Ui, index: usize) -> bool {
        let mut changed = false;
        egui::CollapsingHeader::new(format!("Transform - {}", &self.name)).show(ui, |ui| {
            let pos_changed = self.pos.on_egui(ui, index);
            let rot_changed = self.rot.on_egui(ui, index);

            changed = pos_changed || rot_changed;

            if changed {
                self.model = Transform::model_matrix(self.pos, self.rot);
            }
        });

        changed
    }
}
