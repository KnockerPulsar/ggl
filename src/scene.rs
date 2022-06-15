use std::cell::{RefCell, RefMut};

use egui::{Context, LayerId, Pos2, RawInput, Rect, Ui};
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation};
use egui_glow::EguiGlow;
use glutin::{dpi::PhysicalSize, window::Window, ContextWrapper, PossiblyCurrent};

use crate::{
    camera::Camera,
    egui_drawable::EguiDrawable,
    input::InputSystem,
    light::{DirectionalLight, LightColors, PointLight, SpotLight},
    transform::Transform,
};

pub trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);

    fn draw_egui(&mut self, ui: &mut Ui, entity_id: usize);
}

impl<T: 'static + EguiDrawable> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None);
    }

    fn draw_egui(&mut self, ui: &mut Ui, entity_id: usize) {
        if let Some(comp) = &mut self
            .as_any_mut()
            .downcast_mut::<RefCell<Vec<Option<T>>>>()
            .unwrap()
            .get_mut()[entity_id]
        {
            comp.on_egui(ui, entity_id);
        }
    }
}

pub struct Scene {
    pub component_vecs: Vec<Box<dyn ComponentVec>>,
    pub entity_count: usize,
    pub selected_entity: Option<usize>,
    pub camera: Camera,

    window_width: i32,
    window_height: i32,

    gizmo_mode: GizmoMode,
}

impl Scene {
    pub fn new(window_width: i32, window_height: i32) -> Self {
        let cam = Camera::new(
            &glm::vec3(0.0, 0.0, 2.0f32),
            &glm::vec3(0.0, 1.0, 0.0f32),
            &glm::vec2(0.0, 0.0),
        );

        Scene {
            component_vecs: Vec::new(),
            entity_count: 0,
            selected_entity: None,
            camera: cam,
            window_width: window_width,
            window_height: window_height,
            gizmo_mode: GizmoMode::Translate,
        }
    }

    pub fn add_entity(&mut self) -> usize {
        let e_id = self.entity_count;
        for comp_vec in &mut self.component_vecs {
            comp_vec.push_none();
        }

        self.entity_count += 1;
        e_id
    }

    pub fn add_comp_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) -> &mut Self
    where
        ComponentType: EguiDrawable,
    {
        for comp_vec in self.component_vecs.iter_mut() {
            if let Some(comp_vec) = comp_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                comp_vec.get_mut()[entity] = Some(component);
                return self;
            }
        }

        let mut new_comp_vec: Vec<Option<ComponentType>> = Vec::with_capacity(self.entity_count);

        for _ in 0..self.entity_count {
            new_comp_vec.push(None);
        }

        new_comp_vec[entity] = Some(component);

        self.component_vecs
            .push(Box::new(RefCell::new(new_comp_vec)));

        self
    }

    pub fn borrow_comp_vec<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for comp_vec in self.component_vecs.iter() {
            if let Some(comp_vec) = comp_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(comp_vec.borrow_mut());
            }
        }

        None
    }

    pub fn entity_list(&mut self, egui_ctx: &Context) {
        egui::Window::new("Entities").show(egui_ctx, |ui| {
            for i in 0..self.entity_count {
                ui.group(|ui| {
                    if ui.button(format!("Select entity {}", i)).clicked() {
                        self.selected_entity = Some(i);
                    }

                    for comp_vec in &mut self.component_vecs {
                        comp_vec.draw_egui(ui, i);
                    }
                });
            }
        });
    }

    pub fn selected_gizmo(&mut self, egui_ctx: &Context) {
        egui::Area::new("Gizmo")
            .fixed_pos((0.0, 0.0))
            .show(egui_ctx, |ui| {
                // Draw the gizmo in the background
                ui.with_layer_id(LayerId::background(), |ui| {
                    // If we have a selected entity
                    if let Some(selected_entity_id) = self.selected_entity {
                        // If the entity has a transform
                        if let Some(selected_entity_transform) =
                            &mut self.borrow_comp_vec::<Transform>().unwrap()[selected_entity_id]
                        {
                            let gizmo = Gizmo::new("My gizmo")
                                .view_matrix(self.camera.get_view_matrix())
                                .projection_matrix(self.get_proj_matrix())
                                .model_matrix(selected_entity_transform.get_model_matrix())
                                .mode(self.gizmo_mode)
                                .orientation(GizmoOrientation::Local);

                            if let Some(response) = gizmo.interact(ui) {
                                selected_entity_transform.set_model(response.transform.into());
                            }
                        }
                    }
                })
            });
    }

    pub fn entities_egui(
        &mut self,
        input: &mut InputSystem,
        egui_glow: &mut EguiGlow,
        window: &ContextWrapper<PossiblyCurrent, Window>,
    ) {
        egui_glow.run(window.window(), |egui_ctx| {
            self.entity_list(egui_ctx);
            self.selected_gizmo(egui_ctx);
        });

        if input.is_down(glutin::event::VirtualKeyCode::T) {
            self.gizmo_mode = GizmoMode::Translate;
        }

        if input.is_down(glutin::event::VirtualKeyCode::R) {
            self.gizmo_mode = GizmoMode::Rotate;
        }

        if input.is_down(glutin::event::VirtualKeyCode::Y) {
            self.gizmo_mode = GizmoMode::Scale;
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

impl Scene {
    pub fn light_test(window_width: i32, window_height: i32) -> Self {
        let mut s = Scene::new(window_width, window_height);

        let spot0 = s.add_entity();
        s.add_comp_to_entity(
            spot0,
            Transform::new(
                glm::vec3(3.0, 0.0, 0.0),
                glm::vec3(0.0, 0.0, -90.0),
                "Spotlight 0",
            ),
        )
        .add_comp_to_entity(
            spot0,
            SpotLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.1f32, 0.0, 0.0),
                    diffuse: glm::vec3(10.0, 0.0, 0.0),
                    specular: glm::vec3(0.0, 10.0, 10.0),
                },
                attenuation_constants: glm::vec3(1.0, 0.0, 1.0),
                cutoff_angles: glm::vec2(2.5f32, 5f32),
            },
        );

        let spot1 = s.add_entity();
        s.add_comp_to_entity(
            spot1,
            Transform::new(
                glm::vec3(-3.0, -2.0, -2.0),
                glm::vec3(0.0, -31.0, 115.0),
                "Spotlight 1",
            ),
        )
        .add_comp_to_entity(
            spot1,
            SpotLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.0, 0.0, 0.1f32),
                    diffuse: glm::vec3(0.0, 1.0, 0.0f32),
                    specular: glm::vec3(1.0, 0.0, 0.0),
                },
                attenuation_constants: glm::vec3(0.1, 0.0, 1.0),
                cutoff_angles: glm::vec2(4.0, 10.0),
            },
        );

        let point0 = s.add_entity();
        s.add_comp_to_entity(
            point0,
            Transform::new(
                glm::vec3(0.0, 2.0, 0.0),
                glm::Vec3::zeros(),
                "Point light 0",
            ),
        )
        .add_comp_to_entity(
            point0,
            PointLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.1, 0.03, 0.1),
                    diffuse: glm::vec3(0.7, 0.1, 0.7),
                    specular: glm::vec3(0.5, 0.0, 0.0),
                },
                attenuation_constants: glm::vec3(0.2, 0.0, 0.5),
            },
        );

        let point1 = s.add_entity();
        s.add_comp_to_entity(
            point1,
            Transform::new(
                glm::vec3(0.0, -2.0, 0.0),
                glm::Vec3::zeros(),
                "Point light 1",
            ),
        )
        .add_comp_to_entity(
            point1,
            PointLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.0, 0.0, 0.1),
                    diffuse: glm::vec3(0.0, 0.0, 0.9),
                    specular: glm::vec3(0.0, 1.0, 0.0),
                },
                attenuation_constants: glm::vec3(0.1, 0.0, 1.0),
            },
        );

        let point1 = s.add_entity();
        s.add_comp_to_entity(
            point1,
            Transform::new(
                glm::vec3(0.0, 0.0, 0.0),
                glm::Vec3::zeros(),
                "Directional Light",
            ),
        )
        .add_comp_to_entity(
            point1,
            DirectionalLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.0, 0.0, 0.1),
                    diffuse: glm::vec3(0.0, 0.0, 0.9),
                    specular: glm::vec3(0.0, 1.0, 0.0),
                },
            },
        );

        s
    }
}
