#![allow(dead_code)]

extern crate nalgebra_glm as glm;
use paste::paste;

use std::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
    fmt,
};

use crate::{egui_drawable::EguiDrawable, transform::Transform};
use egui::Ui;

macro_rules! count {
    () => (0usize);
    ( $x:ident $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! addable_component_def {
    ($($comp: ident),+) => {

       /// For use in the editor UI.
       #[derive(Debug, PartialEq, Clone, Copy)]
       pub enum AddableComponent {
          $($comp,)+
       }

       impl fmt::Display for AddableComponent {
           fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
               fmt::Debug::fmt(self, f)
           }
       }

       pub const ADDABLE_COMPONENTS: [AddableComponent; count!($($comp)+)] = [$(AddableComponent::$comp),+];
    };
}

addable_component_def!(Transform, PointLight, SpotLight, DirectionalLight);

#[macro_export]
macro_rules! add_default_component {
    ($ecs: expr, $selected_entity: expr, [$($k:ident),*]) => {
        match $ecs.component_to_add {
            $(
                AddableComponent::$k => $ecs.add_comp_to_entity($selected_entity, $k::default()),
            )*
        };
    };
}

macro_rules! zip {
    ($x: expr) => ($x.iter());
    ($x: expr, $($y: expr), +) => (
        $x.iter().zip(zip!($($y), +))
    )
}

macro_rules! zip_mut {
    ($x: expr) => ($x.iter_mut());
    ($x: expr, $($y: expr), +) => (
        $x.iter_mut().zip(zip_mut!($($y), +))
    )
}

macro_rules! parens {
    ($x: ident) => ($x);
    ($x: ident, $($y: ident), +) => (
        ($x, parens!($($y), +))
    )
}

macro_rules! query_struct {
    ($n: literal, ($($t: tt),*), ($($i: literal),*), ($($p: ident),*)) => {
        paste! {
            pub struct [<Query $n>]<'a, $($t: 'static),*> {
                refs: ($(CompVecRefMut<'a, $t>),*,),
            }

            impl<'a, $($t),*> [<Query $n>]<'a, $($t),*> {
                pub fn iter_mut(&mut self) -> impl Iterator<Item = ($(&mut Option<$t>),*,)> {
                        zip_mut!($(paste!(self.refs.$i.cast_mut())),*)
                            .map(|parens!( $($p),* )| ($($p),*,))
                }
            }

            impl<'a, $($t),*> [<Query $n>]<'a, $($t),*> {
                pub fn iter(&self) -> impl Iterator<Item = ($(&Option<$t>),*,)> {
                        zip!($(paste!(self.refs.$i.cast())),*)
                            .map(|parens!( $($p),* )| ($($p),*,))
                }
            }
        }
    };
}

macro_rules! query {
    ($n: literal, ($($t: tt),*)) => {
        paste! {
            pub fn [<query $n>]<$($t),*>(&self) -> Option<[<Query$n>]<$($t),*>>
                where $($t: 'static),*
            {
                if !($(self.borrow_comp_vec::<$t>().is_some())&&*) {
                    return None;
                }

                Some([<Query $n>] {
                    refs: ($(
                        self.borrow_comp_vec::<$t>().unwrap()
                    ),*,)
                })
            }
        }
    };
}

/// RefCell to allow interior mutability (mutate a component vector of `T` while mutate a compnent
/// vector of `U`).
type CompVec<T> = RefCell<Vec<Option<T>>>;

pub trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn push_none(&self);
    fn draw_egui(&self, ui: &mut Ui, entity_id: usize, ecs: &Ecs);
}

impl<T: 'static + EguiDrawable> ComponentVec for CompVec<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&self) {
        self.borrow_mut().push(None);
    }

    fn draw_egui(&self, ui: &mut Ui, entity_id: usize, ecs: &Ecs) {
        let mut vec = self.borrow_mut();
        let Some(comp) = vec.iter_mut().nth(entity_id).unwrap() else {
            return;
        };

        comp.on_egui(ui, entity_id, ecs);
    }
}

pub struct Ecs {
    entity_count: usize,
    pub component_vecs: Vec<Box<dyn ComponentVec>>,

    // For UI
    // TODO: Probably should separate the renderer and editor more.
    // Will still have to expose a few shared structs (transform, model, etc...) for both the
    // renderer and editor to use. I think the next step after the separation would be to make them
    // traits to allow usability with custom type (i.e. a team using their own `Transform` type can
    // still use the renderer)
    pub component_to_add: AddableComponent,
}

/// Extends the lifetime of the `Vec<Option<T>>` inside it so it lives long enough for queries.
pub struct CompVecRefMut<'a, T: 'static> {
    bar: RefMut<'a, Vec<Option<T>>>,
}

impl<'a, T: 'static> CompVecRefMut<'a, T> {
    pub fn cast(&self) -> &Vec<Option<T>> {
        return self.bar.as_ref();
    }

    pub fn cast_mut(&mut self) -> &mut Vec<Option<T>> {
        return self.bar.as_mut();
    }
}

impl Ecs {
    pub fn new() -> Self {
        Ecs {
            entity_count: 0,
            component_vecs: Vec::new(),
            component_to_add: AddableComponent::Transform,
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

    pub fn find_comp_vec<T: 'static>(&self) -> Option<&Box<dyn ComponentVec>> {
        self.component_vecs
            .iter()
            .find(|cv| cv.as_any().is::<CompVec<T>>())
    }

    pub fn comp_vec_exists<T: 'static>(&self) -> bool {
        self.find_comp_vec::<T>().is_some()
    }

    pub fn add_comp_to_entity<T>(&mut self, entity: usize, component: T) -> &mut Self
    where
        T: 'static + EguiDrawable + Clone,
    {
        if let Some(cv) = self.find_comp_vec::<T>() {
            let mut cv = cv
                .as_any()
                .downcast_ref::<CompVec<T>>()
                .unwrap()
                .borrow_mut();

            let comp = &cv[entity];

            if comp.is_some() {
                let type_name = std::any::type_name::<T>();
                eprintln!(
                    "Attempted to add an duplicate component ({type_name}) onto entity ({entity})"
                );
            } else {
                cv.borrow_mut()[entity] = Some(component);
            }
        } else {
            let mut new_comp_vec: Vec<Option<T>> = vec![None; self.entity_count];
            new_comp_vec.fill(None);

            new_comp_vec[entity] = Some(component);

            self.component_vecs
                .push(Box::new(RefCell::new(new_comp_vec)));
        }

        self
    }

    pub fn borrow_comp_vec<T: 'static>(&self) -> Option<CompVecRefMut<T>> {
        for comp_vec in self.component_vecs.iter() {
            if comp_vec.as_ref().as_any().is::<CompVec<T>>() {
                return Some(CompVecRefMut {
                    bar: comp_vec
                        .as_ref()
                        .as_any()
                        .downcast_ref::<CompVec<T>>()
                        .unwrap()
                        .borrow_mut(),
                });
            }
        }

        None
    }

    pub fn add_empty_entity(&mut self) {
        let num_entities = self.num_entities();
        self.add_entity()
            .with(Transform::with_name(format!("Entity {num_entities}")));
    }

    pub fn num_entities(&self) -> usize {
        self.entity_count
    }

    query!(1, (T));
    query!(2, (T, U));
    query!(3, (T, U, V));
}

query_struct!(1, (T), (0), (x));
query_struct!(2, (T, U), (0, 1), (x, y));
query_struct!(3, (T, U, V), (0, 1, 2), (x, y, z));

pub struct EntityBuilder<'a> {
    entity_id: usize,
    ecs: &'a mut Ecs,
}

impl<'a> EntityBuilder<'a> {
    pub fn with_default<ComponentType>(&mut self) -> &mut Self
    where
        ComponentType: 'static + Default + Clone + EguiDrawable,
    {
        self.ecs
            .add_comp_to_entity(self.entity_id, ComponentType::default());
        self
    }

    pub fn with<ComponentType>(&mut self, comp: ComponentType) -> &mut Self
    where
        ComponentType: 'static + Clone + EguiDrawable,
    {
        self.ecs.add_comp_to_entity(self.entity_id, comp);
        self
    }

    pub fn with_clone<ComponentType>(&mut self, comp: &ComponentType) -> &mut Self
    where
        ComponentType: 'static + Default + Clone + EguiDrawable,
    {
        self.ecs.add_comp_to_entity(self.entity_id, comp.clone());
        self
    }
}
