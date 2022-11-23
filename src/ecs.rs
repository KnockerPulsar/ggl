extern crate nalgebra_glm as glm;

use std::cell::{RefCell, RefMut};

use crate::{egui_drawable::EguiDrawable, light::*, transform::Transform};
use egui::{Context, Ui};

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

pub struct Ecs {
    pub entity_count: usize,
    pub component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl Ecs {
    pub fn new() -> Self {
        Ecs {
            entity_count: 0,
            component_vecs: Vec::new(),
        }
    }

    pub fn add_entity(&mut self) -> EntityBuilder {
        let e_id = self.entity_count;
        for comp_vec in &mut self.component_vecs {
            comp_vec.push_none();
        }

        self.entity_count += 1;
        EntityBuilder {
            entity_id: e_id,
            ecs: self,
        }
    }

    pub fn add_comp_to_entity<ComponentType: 'static + EguiDrawable>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) -> &mut Self {
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

    pub fn entity_list(&mut self, egui_ctx: &Context, selected_entity: Option<usize>) -> Option<usize> {
        let mut just_selected_entity = selected_entity;
        

        egui::Window::new("Entities").show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let transforms = self.borrow_comp_vec::<Transform>().unwrap();
                    for (entity_id, transform) in transforms.iter().enumerate() {
                        if let Some(transform) = transform {
                            if ui.button(transform.get_name()).clicked() {
                                just_selected_entity = Some(entity_id);
                            }
                        }
                    }
                });

                if let Some(selected_entity) = just_selected_entity {
                    let name = {
                        let transforms = self.borrow_comp_vec::<Transform>().unwrap();
                        transforms[selected_entity].as_ref().unwrap().get_name().to_string()
                    };

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(name).strong().underline());

                        for component_vector in &mut self.component_vecs {
                            component_vector.draw_egui(ui, selected_entity);
                        }
                    });
                }
            })
        });

        just_selected_entity
    }
}

impl Ecs {
    pub fn light_test() -> Self {
        let mut ecs = Ecs::new();

        let _spot0 = ecs
            .add_entity()
            .with(Transform::new(
                    glm::vec3(3.0, 0.0, 0.0),
                    glm::vec3(0.0, 0.0, -90.0),
                    "Spotlight 0",
            ))
            .with(SpotLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.1f32, 0.0, 0.0),
                    diffuse: glm::vec3(10.0, 0.0, 0.0),
                    specular: glm::vec3(0.0, 10.0, 10.0),
                },
                attenuation_constants: glm::vec3(1.0, 0.0, 1.0),
                cutoff_angles: glm::vec2(2.5f32, 5f32),
            });

        let _spot1 = ecs
            .add_entity()
            .with(Transform::new(
                    glm::vec3(-3.0, -2.0, -2.0),
                    glm::vec3(0.0, -31.0, 115.0),
                    "Spotlight 1",
            ))
            .with(SpotLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.0, 0.0, 0.1f32),
                    diffuse: glm::vec3(0.0, 1.0, 0.0f32),
                    specular: glm::vec3(1.0, 0.0, 0.0),
                },
                attenuation_constants: glm::vec3(0.1, 0.0, 1.0),
                cutoff_angles: glm::vec2(4.0, 10.0),
            });

        let _point0 = ecs
            .add_entity()
            .with(Transform::new(
                    glm::vec3(0.0, 2.0, 0.0),
                    glm::Vec3::zeros(),
                    "Point light 0",
            ))
            .with(PointLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.1, 0.03, 0.1),
                    diffuse: glm::vec3(0.7, 0.1, 0.7),
                    specular: glm::vec3(0.5, 0.0, 0.0),
                },
                attenuation_constants: glm::vec3(0.2, 0.0, 0.5),
            });

        let _point1 = ecs
            .add_entity()
            .with(Transform::new(
                    glm::vec3(0.0, -2.0, 0.0),
                    glm::Vec3::zeros(),
                    "Point light 1",
            ))
            .with(PointLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.0, 0.0, 0.1),
                    diffuse: glm::vec3(0.0, 0.0, 0.9),
                    specular: glm::vec3(0.0, 1.0, 0.0),
                },
                attenuation_constants: glm::vec3(0.1, 0.0, 1.0),
            });

        let _directional = ecs
            .add_entity()
            .with(Transform::new(
                    glm::vec3(0.0, 0.0, 0.0),
                    glm::Vec3::zeros(),
                    "Directional Light",
            ))
            .with(DirectionalLight {
                enabled: true,
                colors: LightColors {
                    ambient: glm::vec3(0.0, 0.0, 0.1),
                    diffuse: glm::vec3(0.0, 0.0, 0.9),
                    specular: glm::vec3(0.0, 1.0, 0.0),
                },
            });

        ecs
    }
}

pub struct EntityBuilder<'a> {
    entity_id: usize,
    ecs: &'a mut Ecs,
}

impl<'a> EntityBuilder<'a> {
    #[allow(dead_code)]
    pub fn with_default<ComponentType: 'static + Default + EguiDrawable>(&mut self) -> &mut Self {
        self.ecs
            .add_comp_to_entity::<ComponentType>(self.entity_id, ComponentType::default());

        self
    }

    pub fn with<ComponentType: 'static + EguiDrawable>(
        &mut self,
        comp: ComponentType,
    ) -> &mut Self {
        self.ecs
            .add_comp_to_entity::<ComponentType>(self.entity_id, comp);

        self
    }
}
