#![allow(dead_code)]

extern crate nalgebra_glm as glm;

use core::fmt;
use std::{cell::{RefCell, RefMut}, borrow::BorrowMut, fmt::Display, convert::identity};
use crate::{
    egui_drawable::EguiDrawable,
    transform::Transform,
    light::*
};
use egui::Ui;

macro_rules! addable_component_def {
    ($($comp: ident),+) => {
       #[derive(Debug, PartialEq, Clone, Copy)]
       enum AddableComponent {
          $(
            $comp,
          )+ 
       }  

       impl Display for AddableComponent {
           fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
               fmt::Debug::fmt(self, f)
           }
       }
    };
}

macro_rules! addable_components {
    ($($comp: ident),+) => {
        [$(AddableComponent::$comp),+] 
    };
}

macro_rules! add_component {
    ($self: ident, $comp_var: expr, $selected_entity: ident, [$($k:ident),*]) => {
        match $comp_var {
            $(
                AddableComponent::$k => $self.add_comp_to_entity($selected_entity, $k::default()),
            )*
        };
    };
}

addable_component_def!(Transform, PointLight, SpotLight, DirectionalLight);

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
    entity_count: usize,
    pub component_vecs: Vec<Box<dyn ComponentVec>>,
    component_to_add: AddableComponent
}

impl Ecs {
    pub fn new() -> Self {
        Ecs {
            entity_count: 0,
            component_vecs: Vec::new(),
            component_to_add: AddableComponent::Transform
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

    pub fn add_comp_to_entity<ComponentType>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) -> &mut Self 
        where ComponentType: 'static + EguiDrawable + Clone 
    {
        for comp_vec in self.component_vecs.iter_mut() {
            if let Some(comp_vec) = comp_vec
                .as_any_mut()
                    .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                match comp_vec.get_mut()[entity] {
                    Some(_) => {
                        let type_name = std::any::type_name::<ComponentType>();
                        eprintln!("Attempted to add an duplicate component ({type_name}) onto entity ({entity})")
                    },
                    None => comp_vec.get_mut()[entity] = Some(component),
                }
                return self;
            }
        }

        let mut new_comp_vec: Vec<Option<ComponentType>> = vec![None; self.entity_count];
        new_comp_vec.fill(None);

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

    pub fn entity_list(&mut self, ui: &mut Ui, selected_entity: Option<usize>) -> Option<usize> {
        let mut just_selected_entity = selected_entity;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.do_all_some::<Transform>(|(id, transform)| {
                    if ui.button(transform.get_name()).clicked() {
                        just_selected_entity = Some(id);
                    }
                });

            });

            if let Some(selected_entity) = just_selected_entity {
                let name = self.do_entity::<Transform, String>(selected_entity, |t| {
                    t.get_name().into()
                });

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Select one!")
                            .selected_text(self.component_to_add.to_string())
                            .show_ui(ui, |ui| {
                                for addable_component in addable_components![Transform, PointLight, SpotLight, DirectionalLight] {
                                    ui.selectable_value(&mut self.component_to_add, addable_component, addable_component.to_string());
                                }
                            });

                        if ui.button("Add component").clicked() {
                            add_component! {
                                self, self.component_to_add, selected_entity,
                                [Transform, PointLight, SpotLight, DirectionalLight]
                            }
                        }
                    
                    });

                    ui.add_space(10.);

                    ui.label(egui::RichText::new(name).strong().underline());

                    for component_vector in &mut self.component_vecs {
                        component_vector.draw_egui(ui, selected_entity);
                    }

                });
            }
        });

        just_selected_entity
    }

    pub fn do_n<T , U>(&self, mut f: impl FnMut(&mut T, &mut U), n: usize) 
        where T: 'static, U: 'static
    {
        let t = self.borrow_comp_vec::<T>();
        let u = self.borrow_comp_vec::<U>();

        // if t.is_none() {
        //     println!("do_all: Component type {:?} not found", std::any::type_name::<T>());
        // }
        //
        // if u.is_none() {
        //     println!("do_all: Component type {:?} not found", std::any::type_name::<U>());
        // }

        if t.is_none() || u.is_none() {
            return;
        }

        let mut t = t.unwrap();
        let mut u = u.unwrap();

        t 
            .iter_mut()
            .zip(u.iter_mut())
            .filter(|(x,y)| x.is_some() && y.is_some())
            .map(|(x,y)| (x.as_mut().unwrap(), y.as_mut().unwrap()))
            .take(n)
            .for_each(|(x,y)| f(x, y));
    }

    pub fn do_all<T , U>(&self, f: impl FnMut(&mut T, &mut U)) 
        where T: 'static, U: 'static 
    {
        self.do_n::<T,U>(f, self.entity_count);
    }

    pub fn do_one<T , U>(&self, f: impl FnMut(&mut T, &mut U)) 
        where T: 'static, U: 'static 
    {
        self.do_n::<T,U>(f, 1);
    }

    pub fn do_entity<T, U>(&self, entity_id: usize, mut f: impl (FnMut(&mut T) -> U)) -> U
        where T: 'static 
    {
        assert!( entity_id < self.entity_count );

        let mut comp_vec = self.borrow_comp_vec::<T>()
            .unwrap();

        let mut comp = comp_vec
            .borrow_mut()[entity_id]
            .as_mut().unwrap();

        f(&mut comp)
    }

    pub fn num_entities(&self) -> usize {
       self.entity_count 
    }

    pub fn do_all_some<T>(&self, f: impl FnMut((usize, &mut T))) where T: 'static {
        self.borrow_comp_vec::<T>()
            .unwrap()
            .iter_mut()
            .enumerate()
            .filter_map(|(id, it)| {
                match it {
                    Some(it) => Some((id, it)),
                    None => None,
                }
            })
            .for_each(f);
    }
}


pub struct EntityBuilder<'a> {
    entity_id: usize,
    ecs: &'a mut Ecs,
}

impl<'a> EntityBuilder<'a> {
    pub fn with_default<ComponentType>(&mut self) -> &mut Self 
        where ComponentType: 'static + Default + Clone + EguiDrawable
    {
        self.ecs
            .add_comp_to_entity::<ComponentType>(self.entity_id, ComponentType::default());

        self
    }

    pub fn with<ComponentType>(
        &mut self,
        comp: ComponentType,
    ) -> &mut Self 
        where ComponentType: 'static + Default + Clone + EguiDrawable
    {
        self.ecs
            .add_comp_to_entity::<ComponentType>(self.entity_id, comp);

        self
    }

    pub fn with_clone<ComponentType>(
        &mut self,
        comp: &ComponentType,
    ) -> &mut Self 
        where ComponentType: 'static + Default + Clone + EguiDrawable
    {
        self.ecs
            .add_comp_to_entity::<ComponentType>(self.entity_id, comp.clone());

        self
    }
}
