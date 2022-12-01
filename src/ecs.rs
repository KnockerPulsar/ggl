extern crate nalgebra_glm as glm;

use std::{cell::{RefCell, RefMut}, borrow::BorrowMut};
use crate::{
    egui_drawable::EguiDrawable,
    light::*,
    transform::{Transform, Degree3},
    obj_loader::{Model, ObjLoader, ModelHandle},
    asset_loader::TextureLoader,
    texture::Texture2D
};
use egui::Ui;
use nalgebra_glm::{vec2, vec3, Vec3};

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

    pub fn entity_list(&mut self, ui: &mut Ui, selected_entity: Option<usize>) -> Option<usize> {
        let mut just_selected_entity = selected_entity;
        

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

    pub fn do_entity<T>(&self, entity_id: usize, f: impl FnMut(&mut T)) 
        where T: 'static 
    {
        assert!( entity_id < self.entity_count );

        self.borrow_comp_vec::<T>()
            .unwrap()
            .borrow_mut()
            .iter_mut()
            .skip(entity_id)
            .take(1)
            .filter_map(|x| x.as_mut())
            .for_each(f);
    }
}

impl Ecs {
    pub fn light_test(texture_loader: &mut TextureLoader, object_loader: &mut ObjLoader) -> Self {
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

        let cube_path = "assets/obj/cube.obj";
        let checker_path = "assets/textures/checker_32_32.jpg";
        let white_path = "assets/textures/white.jpeg";

        let cube_handle = {
            let cube_handle = object_loader.load(cube_path, texture_loader).unwrap();

            let cube_model = object_loader.borrow(&cube_handle).unwrap();
            let checker_texture = texture_loader.load_into_handle(checker_path).unwrap();
            let white = texture_loader.load_into_handle(white_path).unwrap();

            cube_model.add_texture( &Texture2D { 
                handle: checker_texture, 
                tex_type: crate::texture::TextureType::Diffuse
            }).add_texture( &Texture2D { 
                handle: white,
                tex_type: crate::texture::TextureType::Specular
            });

            cube_handle
        };

        let _spot0 = ecs
            .add_entity()
            .with(Transform::new(
                    positions[2] + up_two,
                    Degree3::xyz(0.0, 0.0, 0.),
                    "Spotlight 0",
            ))
            .with(SpotLight {
                enabled: true,
                colors: LightColors::no_ambient(vec3(0., 3., 3.), 0.1),
                attenuation_constants: vec3(0.1, 0.3, 0.),
                cutoff_angles: vec2(10f32, 15f32),
            });

        let _spot1 = ecs
            .add_entity()
            .with(Transform::new(
                    vec3(-3.4, 1.7, 3.7),
                    Degree3::xyz(-70., -25., -50.),
                    "Spotlight 1",
            ))
            .with(SpotLight {
                enabled: true,
                colors: LightColors::no_ambient(vec3(1., 1., 1.), 0.7),
                attenuation_constants: vec3(0.1, 0.0, 1.0),
                cutoff_angles: vec2(20., 30.),
            });
        
        let _point0 = ecs
            .add_entity()
            .with(Transform::new(
                    positions[0] + up_two,
                    Degree3::default(),
                    "Point light 0",
            ))
            .with(PointLight {
                enabled: true,
                colors:  LightColors::no_ambient(vec3(2., 0., 0.), 0.1),
                attenuation_constants: vec3(0.2, 0.0, 0.5),
            });
        
        let _point1 = ecs
            .add_entity()
            .with(Transform::new(
                    positions[1] + up_two,
                    Degree3::default(),
                    "Point light 1",
            ))
            .with(PointLight {
                enabled: true,
                colors: LightColors::no_ambient(vec3(0., 1., 0.), 0.07),
                attenuation_constants: vec3(0.1, 0.0, 1.0),
            });
        
        // let _directional = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             vec3(0.0, 0.0, 0.0),
        //             Vec3::zeros(),
        //             "Directional Light",
        //     ))
        //     .with(DirectionalLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(0., 1., 0.), 0.9),
        //     });


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

        ecs
    }
}

pub struct EntityBuilder<'a> {
    entity_id: usize,
    ecs: &'a mut Ecs,
}

impl<'a> EntityBuilder<'a> {
    #[allow(dead_code)]
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
