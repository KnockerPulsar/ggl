#![allow(dead_code)]

extern crate nalgebra_glm as glm;

use core::fmt;

use std::{
    cell::{RefCell, RefMut},
    borrow::BorrowMut,
    fmt::Display, convert::identity
};

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

macro_rules! type_vec_mut {
    ($self: expr, $type: ty) => {
        $self.as_any_mut().downcast_mut::<RefCell<CompVec<$type>>>()
    }
}

macro_rules! type_vec_ref {
    ($self: expr, $type: ty) => {
        $self.as_any().downcast_ref::<RefCell<CompVec<$type>>>()
    }
}

macro_rules! entity_comp {
    ($comp_vec: ident, $entity_id: expr) => {
        $comp_vec.get_mut()[$entity_id].as_mut()
    }
}

addable_component_def!(Transform, PointLight, SpotLight, DirectionalLight);

type CompVec<T> = Vec<Option<T>>;

pub trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);

    fn draw_egui(&mut self, ui: &mut Ui, entity_id: usize);
}

impl<T: 'static + EguiDrawable> ComponentVec for RefCell<CompVec<T>> {
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
        let Some(comp_vec) = &mut type_vec_mut!(self, T) else { return; };
        let Some(comp) = entity_comp!(comp_vec, entity_id) else { return };

        comp.on_egui(ui, entity_id);
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

    pub fn add_comp_to_entity<T>(
        &mut self,
        entity: usize,
        component: T,
    ) -> &mut Self 
        where T: 'static + EguiDrawable + Clone 
    {
        for comp_vec in self.component_vecs.iter_mut() {
            let Some(comp_vec) = type_vec_mut!(comp_vec, T) else { continue; };

            if entity_comp!(comp_vec, entity).is_some() {
                let type_name = std::any::type_name::<T>();
                eprintln!("Attempted to add an duplicate component ({type_name}) onto entity ({entity})")
            }

            comp_vec.get_mut()[entity] = Some(component);
            return self;
        }

        let mut new_comp_vec: Vec<Option<T>> = vec![None; self.entity_count];
        new_comp_vec.fill(None);

        new_comp_vec[entity] = Some(component);

        self.component_vecs
            .push(Box::new(RefCell::new(new_comp_vec)));

        self
    }

    pub fn borrow_comp_vec<T: 'static>( &self,) -> Option<RefMut<CompVec<T>>> {
        for comp_vec in self.component_vecs.iter() {
            if let Some(comp_vec) =  type_vec_ref!(comp_vec, T) {
                return Some(comp_vec.borrow_mut());
            }
        }

        None
    }

    fn add_component_ui(&mut self, ui: &mut Ui, selected_entity: usize) {
        const ADDABLE_COMPONENTS: [AddableComponent; 4] = addable_components![Transform, PointLight, SpotLight, DirectionalLight];
        ui.horizontal(|ui| {
            
            let combobox = egui::ComboBox::from_label("Select one!")
                .selected_text(self.component_to_add.to_string());

            // Changed component to be potentially added
            combobox.show_ui(ui, |ui| {
                ADDABLE_COMPONENTS.map(|comp| {
                    ui.selectable_value(&mut self.component_to_add, comp, comp.to_string());
                })
            });

            // Add the currently seleccted component type
            if ui.button("Add component").clicked() {
                add_component! {
                    self, self.component_to_add, selected_entity,
                    [Transform, PointLight, SpotLight, DirectionalLight]
                }
            }

        });
    }

    /// Show a list of buttons where the label is the entity's name.
    /// Returns Some(id) if an entity was clicked. None otherwise.
    fn select_an_entity(&mut self, ui: &mut Ui) -> Option<usize> {
        ui.vertical(|ui| {
            let clicked = self.do_all_some::<Transform, usize>(|(id, transform)| {
                if ui.button(transform.get_name()).clicked() {
                    Some(id)
                } else {
                    None
                }
            });

            // Shouldn't click more than one button
            clicked.first().copied()
        }).inner
    }

    pub fn entity_list(&mut self, ui: &mut Ui, prev_selected: Option<usize>) -> Option<usize> {
        ui.horizontal(|ui| {
            let new_selected = self.select_an_entity(ui);

            let selection = match (new_selected, prev_selected) {
                (None, None) => None,
                (None, Some(_)) => prev_selected,
                (Some(_), _) => new_selected,
            }; 

            let Some(selection) = selection else { return None; };

            let name = self.do_entity::<Transform, String>(selection, |t| { t.get_name().into() });



            ui.vertical(|ui| {

                ui.vertical_centered(|ui| { ui.label(egui::RichText::new(name).strong().underline().size(20.)); });

                ui.add_space(10.);


                self.component_vecs
                    .iter_mut()
                    .for_each(|cv| cv.draw_egui(ui, selection));

                ui.add_space(10.);

                self.add_component_ui(ui, selection);

            });

            Some(selection)
        }).inner
    }

    pub fn do_n<T , U>(&self, mut f: impl FnMut(&mut T, &mut U), n: usize) 
        where T: 'static, U: 'static
    {
        let (Some(mut t), Some(mut u)) = (self.borrow_comp_vec::<T>(), self.borrow_comp_vec::<U>()) else {
            // use std::any::type_name;
            // println!("do_all: Component type {:?} or {:?} not found", type_name::<T>(), type_name::<U>());
            return;
        };

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


    pub fn do_all_some<T, U>(&self, f: impl (FnMut((usize, &mut T))-> Option<U>)) -> Vec<U>
        where T: 'static {
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
            .map(f)
            .filter_map(identity)
            .collect()
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
        self.ecs .add_comp_to_entity(self.entity_id, ComponentType::default());
        self
    }

    pub fn with<ComponentType>(
        &mut self,
        comp: ComponentType,
    ) -> &mut Self 
        where ComponentType: 'static + Default + Clone + EguiDrawable
    {
        self.ecs .add_comp_to_entity(self.entity_id, comp);
        self
    }

    pub fn with_clone<ComponentType>(
        &mut self,
        comp: &ComponentType,
    ) -> &mut Self 
        where ComponentType: 'static + Default + Clone + EguiDrawable
    {
        self.ecs .add_comp_to_entity(self.entity_id, comp.clone());
        self
    }
}

