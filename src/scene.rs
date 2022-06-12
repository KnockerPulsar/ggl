use std::cell::{RefCell, RefMut};

use egui::Ui;

use crate::egui_drawable::EguiDrawable;


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
            comp.on_egui(ui);
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
}

impl EguiDrawable for Scene {
    fn on_egui(&mut self, ui: &mut egui::Ui) {
        for i in 0..self.entity_count {
            ui.group(|ui| {
                for comp_vec in &mut self.component_vecs {
                    comp_vec.draw_egui(ui, i);
                }
            });
            ui.separator();
        }
    }
}
