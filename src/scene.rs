extern crate nalgebra_glm as glm;

use egui::{Context, LayerId, Ui};
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation};
use glutin::dpi::PhysicalSize;

use crate::{camera::Camera, ecs::Ecs, input::InputSystem, transform::Transform};

pub struct Scene {
    pub selected_entity: Option<usize>,
    pub camera: Camera,

    window_width: i32,
    window_height: i32,

    pub gizmo_mode: GizmoMode,
}

impl Scene {
    pub fn new(window_width: i32, window_height: i32) -> Self {
        let cam = Camera::new(
            &glm::vec3(0.0, 0.0, 2.0f32),
            &glm::vec3(0.0, 1.0, 0.0f32),
            &glm::vec2(0.0, 0.0),
        );

        Scene {
            selected_entity: None,
            camera: cam,
            window_width,
            window_height,
            gizmo_mode: GizmoMode::Translate,
        }
    }

    pub fn selected_entity_gizmo(&mut self, egui_ctx: &Context, ecs: &mut Ecs) {
        egui::Area::new("Gizmo")
            .fixed_pos((0.0, 0.0))
            .show(egui_ctx, |ui| {
                // Draw the gizmo in the background
                ui.with_layer_id(LayerId::background(), |ui| {
                    // If we have a selected entity
                    if let Some(selected_entity_id) = self.selected_entity {
                        // If the entity has a transform
                        ecs.do_entity(selected_entity_id, |selected_entity_transform: &mut Transform| {
                            let gizmo = Gizmo::new("My gizmo")
                                .view_matrix(self.camera.get_view_matrix())
                                .projection_matrix(self.get_proj_matrix())
                                .model_matrix(selected_entity_transform.get_model_matrix())
                                .mode(self.gizmo_mode)
                                .orientation(GizmoOrientation::Local);

                            if let Some(response) = gizmo.interact(ui) {
                                selected_entity_transform.set_model(response.transform.into());
                            }
                        })
                    }
                })
            });
    }

    pub fn entities_egui(
        &mut self,
        ui: &mut Ui,
        ecs: &mut Ecs,
    ) {
        let just_selected_entity = ecs.entity_list(ui, self.selected_entity); 
        if just_selected_entity.is_some() {
            self.selected_entity = just_selected_entity;
        }
    }

    pub fn get_proj_matrix(&self) -> glm::Mat4 {
        glm::perspective(

            self.window_width as f32 / self.window_height as f32,
            self.camera.get_fov_euler().to_radians(),
            0.01,
            1000.0,
        )
    }

    pub fn window_size_changed(&mut self, inner_size: &PhysicalSize<u32>) {
        self.window_width = inner_size.width as i32;
        self.window_height = inner_size.height as i32;
    }
}
