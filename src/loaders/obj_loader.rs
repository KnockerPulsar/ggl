#![allow(dead_code)]

extern crate byteorder;
extern crate itertools;
extern crate obj;

use std::collections::HashMap;

use obj::Obj;

use crate::{
    loaders::*,
    map,
    mesh::Mesh,
    model::{Model, ObjLoadError},
    renderer::Material,
};

use self::utils::Handle;

pub mod utils {
    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    use crate::{ecs::Ecs, egui_drawable::EguiDrawable};

    pub struct Handle<T>(Rc<RefCell<T>>);

    impl<T> Handle<T> {
        pub fn new(t: T) -> Self {
            Handle(Rc::new(RefCell::new(t)))
        }
    }

    impl<T> Deref for Handle<T> {
        type Target = RefCell<T>;

        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }
    }

    impl<T: Clone> Clone for Handle<T> {
        fn clone(&self) -> Self {
            Handle(Rc::clone(&self.0))
        }
    }

    impl<T: EguiDrawable> EguiDrawable for Handle<T> {
        fn on_egui(&mut self, ui: &mut egui::Ui, index: usize, ecs: &Ecs) -> bool {
            self.0
                .deref()
                .borrow_mut()
                .deref_mut()
                .on_egui(ui, index, ecs)
        }
    }
}

pub type ModelLoadResult<T> = std::result::Result<T, ObjLoadError>;

pub const DEFAULT_CUBE_NAME: &str = "default_cube";
pub const DEFAULT_PLANE_NAME: &str = "default_plane";

pub struct ObjLoader {
    models: HashMap<String, Handle<Model>>,
}

impl ObjLoader {
    fn load_default_cube(
        &mut self,
        shader_loader: &mut ShaderLoader,
        texture_loader: &mut TextureLoader,
    ) {
        let cube_path = "assets/obj/cube.obj";

        let cube_model = self.load_obj(DEFAULT_CUBE_NAME, cube_path).unwrap();

        cube_model.borrow_mut().material =
            Some(Material::default_lit(shader_loader, texture_loader));
    }

    fn load_default_plane(
        &mut self,
        shader_loader: &mut ShaderLoader,
        texture_loader: &mut TextureLoader,
    ) {
        //                   ^
        //  (-0.5, 0.5, 0)   |       (0.5, 0.5, 0)
        //      0            |             1
        //                   |
        //                   |
        //                   |
        //                   |
        //                   |
        // ------------------|--------------------->
        //                   |
        //                   |
        //                   |
        //                   |
        //      2            |             3
        // (-0.5, -0.5, 0)   |       (0.5, -0.5, 0)
        //                   |
        let positions = vec![-0.5, 0.5, 0., 0.5, 0.5, 0., -0.5, -0.5, 0., 0.5, -0.5, 0.];

        let normals = vec![
            0., 0., 1., // Normal
            0., 0., 1., 0., 0., 1., 0., 0., 1.,
        ];

        let texture_coordinates = vec![0., 0., 1., 0., 0., 1., 1., 1.];

        let indices: Vec<u32> = vec![2, 0, 1, 2, 1, 3];

        let mat = Material::default_billboard(shader_loader, texture_loader);
        let mesh = Mesh::new(positions, normals, texture_coordinates, indices);
        self.add_model(
            DEFAULT_PLANE_NAME,
            Model::new(DEFAULT_PLANE_NAME, "", vec![mesh], Some(mat)),
        );
    }

    pub fn new(shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) -> Self {
        let mut loader = ObjLoader { models: map! {} };

        loader.load_default_cube(shader_loader, texture_loader);
        loader.load_default_plane(shader_loader, texture_loader);

        loader
    }

    fn add_model(&mut self, model_key: impl Into<String>, model: Model) -> Handle<Model> {
        let model_key = model_key.into();
        self.models.insert(model_key.clone(), Handle::new(model));
        Handle::clone(self.models.get(&model_key).unwrap())
    }

    pub fn load_model(
        &mut self,
        name: impl Into<String>,
        path: impl Into<String>,
    ) -> ModelLoadResult<Handle<Model>> {
        let path_string: String = path.into();
        let name: String = name.into();

        if !self.models.contains_key(&path_string) {
            self.load_obj(name.clone(), &path_string)?;
        }

        let t = self.models.get(&name).unwrap();
        Ok(Handle::clone(t)) // Should always succeed
    }

    pub fn clone_handle(&mut self, model_key: &str) -> Handle<Model> {
        Handle::clone(self.models.get(model_key).unwrap())
    }

    pub fn clone(&mut self, old_name: &str, new_name: &str) -> Handle<Model> {
        assert!(
            new_name != old_name,
            "New and old model names must be different!"
        );

        // Check if old model exists
        //  if so:
        //      clone it
        //  else:
        //      clone the default cube
        //
        //   add the clone under the new name
        //   return the clone's reference

        let clone_name = if self.models.contains_key(old_name) {
            old_name
        } else {
            DEFAULT_CUBE_NAME
        };

        let old_rc = self.models.get(clone_name).unwrap();

        let mut model_clone: Model = (*old_rc).borrow().clone();
        model_clone.name = new_name.into();

        self.add_model(new_name, model_clone)
    }

    pub fn models(&mut self) -> &mut HashMap<String, Handle<Model>> {
        &mut self.models
    }

    // default_model_getters![
    //     (DEFAULT_CUBE_NAME, default_cube_model),
    //     (DEFAULT_PLANE_NAME, default_plane_model)
    // ];
}

impl ObjLoader {
    pub fn load_obj(
        &mut self,
        name: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Handle<Model>, ObjLoadError> {
        let mut objects = Obj::load(path.into())?;

        // Must run this for materials to properly load
        objects.load_mtls().unwrap();

        let dir = objects.path;
        let name: String = name.into();

        let all_pos = objects.data.position;
        let all_norm = objects.data.normal;
        let all_tex = objects.data.texture;

        let mut model = {
            let dir = String::from(dir.to_str().unwrap());
            Model::new(dir.clone(), dir, Vec::new(), None)
        };

        let num_objects = objects.data.objects.len() as f32;

        for (object_index, object) in objects.data.objects.iter().enumerate() {
            println!("Loading object {i}/{num_objects}", i = object_index);

            let obj_group = &object.groups[0];

            let mut positions: Vec<f32> = vec![];
            let mut normals: Vec<f32> = vec![];
            let mut texture_coordinates: Vec<f32> = vec![];
            let mut inds: Vec<u32> = vec![];

            let mut index = 0;
            for poly in obj_group.polys.iter() {
                for vertex in &poly.0 {
                    let pos_index = vertex.0;

                    positions.extend(all_pos[pos_index]);

                    if let Some(tex_index) = vertex.1 {
                        texture_coordinates.extend(all_tex[tex_index]);
                    }

                    if let Some(norm_index) = vertex.2 {
                        normals.extend(all_norm[norm_index]);
                    }

                    inds.push(index);
                    index += 1;
                }
            }

            model
                .meshes
                .push(Mesh::new(positions, normals, texture_coordinates, inds))
        }

        Ok(self.add_model(name, model))
    }
}
