extern crate nalgebra_glm as glm;

use std::path::Path;

use egui::{Context, LayerId, Ui};
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation};
use glm::{vec3, vec2, Vec3};
use glutin::dpi::PhysicalSize;

use crate::{
    camera::Camera,
    ecs::Ecs,
    transform::{Transform, Degree3},
    asset_loader::TextureLoader,
    obj_loader::ObjLoader,
    texture::{Texture2D, TextureType},
    light::{
        SpotLight,
        LightColors,
        PointLight,
        DirectionalLight
    }
};

pub struct Scene {
    pub selected_entity: Option<usize>,
    pub camera: Camera,
    pub gizmo_mode: GizmoMode,
    pub ecs: Ecs
}

impl Scene {
    pub fn empty(window_width: i32, window_height: i32) -> Self {
        let aspect = window_width as f32 / window_height as f32;
        Scene {
            selected_entity: None,
            camera: Camera::new(Vec3::zeros(), vec3(0., 1., 0.), vec2(0., 0.), aspect),
            gizmo_mode: GizmoMode::Translate,
            ecs: Ecs::new()
        }
    }

    pub fn light_test(
        window_width: i32, 
        window_height: i32,
        texture_loader: &mut TextureLoader, 
        object_loader: &mut ObjLoader
    ) -> Self {

        let mut ecs = Ecs::new();
        let positions = [
            vec3(5., 0., 5.),
            vec3(-5., 0., -5.),
            vec3(5., 0., -5.),
            vec3(-5., 0., 5.),
        ];

        let cube_data: Vec<Transform> = positions
            .iter()
            .enumerate()
            .map(|(index, pos)| {
                Transform::new(*pos, Degree3::default(), &format!("cube {}", index))
            })
            .collect();

        let up_two = vec3(0., 2., 0.);
        let cube_handle = object_loader.load("default_cube", texture_loader).unwrap();

        // let _spot0 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             positions[2] + up_two,
        //             Degree3::xyz(0.0, 0.0, 0.),
        //             "Spotlight 0",
        //     ))
        //     .with(SpotLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(0., 3., 3.), 0.1),
        //         attenuation_constants: vec3(0.1, 0.3, 0.),
        //         cutoff_angles: vec2(10f32, 15f32),
        //     });
        //
        // let _spot1 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             vec3(-3.4, 1.7, 3.7),
        //             Degree3::xyz(-70., -25., -50.),
        //             "Spotlight 1",
        //     ))
        //     .with(SpotLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(1., 1., 1.), 0.7),
        //         attenuation_constants: vec3(0.1, 0.0, 1.0),
        //         cutoff_angles: vec2(20., 30.),
        //     });
        //
        // let _point0 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             positions[0] + up_two,
        //             Degree3::default(),
        //             "Point light 0",
        //     ))
        //     .with(PointLight {
        //         enabled: true,
        //         colors:  LightColors::no_ambient(vec3(2., 0., 0.), 0.1),
        //         attenuation_constants: vec3(0.2, 0.0, 0.5),
        //     });
        //
        // let _point1 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             positions[1] + up_two,
        //             Degree3::default(),
        //             "Point light 1",
        //     ))
        //     .with(PointLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(0., 1., 0.), 0.07),
        //         attenuation_constants: vec3(0.1, 0.0, 1.0),
        //     });
        
        let _directional = ecs
            .add_entity()
            .with(Transform::new(
                    vec3(0.0, 0.0, 0.0),
                    Degree3(vec3(0., 0., 0.)),
                    "Directional Light",
            ))
            .with(DirectionalLight {
                enabled: true,
                colors: LightColors::default().ambient(vec3(0.1, 0.04, 0.1)).diffuse(vec3(0.5, 0.2, 0.5)),
            });


        let _ground = ecs
            .add_entity()
            .with(
                Transform::with_scale(
                    vec3(0., -2., 0.),
                    Degree3::xyz(0., 0., 0.), 
                    vec3(10., 1., 10.), 
                    "ground"
                )
            ).with(cube_handle.clone());


        for cube_transform in cube_data {
            let _model = ecs
                .add_entity()
                .with(cube_transform)
                .with(cube_handle.clone());
            }

        let cam = Camera::new(
            glm::vec3(0.0, 1.0, 5.0f32),
            glm::vec3(0.0, 1.0, 0.0f32),
            glm::vec2(0.0, 0.0),
            window_width as f32 / window_height as f32
        );

        let square_handle =  object_loader.load("default_square", texture_loader).unwrap();
        let _billboard_test = ecs.add_entity().with_default::<Transform>().with(square_handle.clone());

        Scene {
            selected_entity: None,
            camera: cam,
            gizmo_mode: GizmoMode::Translate,
            ecs
        }

    }

    pub fn selected_entity_gizmo(&mut self, egui_ctx: &Context) {
        egui::Area::new("Gizmo")
            .fixed_pos((0.0, 0.0))
            .show(egui_ctx, |ui| {
                // Draw the gizmo in the background
                ui.with_layer_id(LayerId::background(), |ui| {
                    // If we have a selected entity
                    if let Some(eid) = self.selected_entity {
                        // If the entity has a transform
                        self.ecs.do_entity(eid, |selected_entity_transform: &mut Transform| {
                            let gizmo = Gizmo::new("My gizmo")
                                .view_matrix(self.camera.get_view_matrix())
                                .projection_matrix(self.camera.get_proj_matrix())
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
    ) {
        let just_selected_entity = self.ecs.entity_list(ui, self.selected_entity); 
        if just_selected_entity.is_some() {
            self.selected_entity = just_selected_entity;
        }
    }

    pub fn window_size_changed(&mut self, inner_size: &PhysicalSize<u32>) {
        self.camera.update_aspect_ratio(inner_size.width as f32, inner_size.height as f32);
    }
}

