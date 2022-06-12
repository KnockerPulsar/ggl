use std::cell::{RefCell, RefMut};

use egui::{RawInput, Ui};
use egui_glow::EguiGlow;
use glutin::{window::Window, ContextWrapper, PossiblyCurrent};

use crate::{
    egui_drawable::EguiDrawable,
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
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            component_vecs: Vec::new(),
            entity_count: 0,
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

    pub fn entities_egui(
        &mut self,
        egui_glow: &mut EguiGlow,
        window: &ContextWrapper<PossiblyCurrent, Window>,
    ) {
        egui_glow.run(window.window(), |egui_ctx| {
            egui::Window::new("Entities").show(egui_ctx, |ui| {
                for i in 0..self.entity_count {
                    ui.group(|ui| {
                        for comp_vec in &mut self.component_vecs {
                            comp_vec.draw_egui(ui, i);
                        }
                    });
                }
            });
        });
    }
}

impl Scene {
    pub fn light_test() -> Self {
        let mut s = Scene::new();

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
