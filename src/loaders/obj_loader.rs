#![allow(dead_code)]

extern crate byteorder;
extern crate itertools;
extern crate obj;

use std::collections::HashMap;
use std::hash::Hash;

use nalgebra_glm::{cross, make_vec3, normalize, Vec3};
use obj::{Group, IndexTuple, Obj, SimplePolygon};

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

            let positions = {
                let position_indices =
                    Self::extract_attribute_indices(obj_group, |poly: &IndexTuple| poly.0);

                position_indices
                    .iter()
                    .map(|index| all_pos[*index])
                    .flatten()
                    .collect::<Vec<_>>()
            };

            let texture_coordinates = {
                let texture_coordinate_indices =
                    Self::extract_attribute_indices(obj_group, |poly: &IndexTuple| poly.1);
                let has_texture_coordinates =
                    texture_coordinate_indices.iter().all(Option::is_some);

                if has_texture_coordinates {
                    texture_coordinate_indices
                        .iter()
                        .map(|index| all_tex[index.unwrap()])
                        .flatten()
                        .collect::<Vec<f32>>()
                } else {
                    vec![]
                }
            };

            let normals = {
                let normal_indices =
                    Self::extract_attribute_indices(obj_group, |poly: &IndexTuple| poly.2);

                let has_normals = normal_indices.iter().all(Option::is_some);
                if has_normals {
                    normal_indices
                        .iter()
                        .map(|index| all_norm[index.unwrap()])
                        .flatten()
                        .collect::<Vec<f32>>()
                } else {
                    Self::flat_shade(&positions)
                }
            };

            let inds = (0u32..positions.len() as u32).collect();

            model
                .meshes
                .push(Mesh::new(positions, normals, texture_coordinates, inds))
        }

        Ok(self.add_model(name, model))
    }

    fn smooth_shade(positions: &[f32]) -> Vec<f32> {
        // For each polygon (triangle):
        //  Get the three vertex positions
        //  Compute v0v1 and v0v2
        //  Compute the cross product
        //  normal = normalize(cross_produdct)
        //  Record that v0, v1, v2 have normal = normal
        //
        // For each vertex:
        //  Compute its average normal from the list of recorded normals
        //  Set the value of the vertex normal to the average

        #[derive(Clone, Copy)]
        struct HashableFloat3 {
            data: [f32; 3],
        }

        impl PartialEq for HashableFloat3 {
            fn eq(&self, other: &Self) -> bool {
                self.data.iter().zip(other.data.iter()).all(|(a, b)| a == b)
            }
        }

        impl Eq for HashableFloat3 {}

        impl Hash for HashableFloat3 {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.data
                    .iter()
                    .for_each(|x| state.write_u32(unsafe { std::mem::transmute(*x) }))
            }
        }

        impl From<&[f32]> for HashableFloat3 {
            fn from(v: &[f32]) -> Self {
                HashableFloat3 {
                    data: v.try_into().unwrap(),
                }
            }
        }

        impl From<Vec3> for HashableFloat3 {
            fn from(v: Vec3) -> Self {
                HashableFloat3 {
                    data: [v.x, v.y, v.z],
                }
            }
        }

        let mut vertex_to_normals: HashMap<HashableFloat3, Vec<Vec3>> = HashMap::new();

        positions.chunks_exact(9).for_each(|verts| {
            let (v0, v1, v2) = (
                make_vec3(&verts[0..3]),
                make_vec3(&verts[3..6]),
                make_vec3(&verts[6..9]),
            );

            let v0v1 = v1 - v0;
            let v0v2 = v2 - v0;

            let n: Vec3 = normalize(&cross(&v0v1, &v0v2));

            vertex_to_normals.entry(v0.into()).or_default().push(n);
            vertex_to_normals.entry(v1.into()).or_default().push(n);
            vertex_to_normals.entry(v2.into()).or_default().push(n);
        });

        let vertex_to_avg_normal = vertex_to_normals
            .iter()
            .map(|(v, ns)| (*v, ns.iter().sum::<Vec3>() / ns.len() as f32))
            .collect::<HashMap<HashableFloat3, Vec3>>();

        positions
            .chunks_exact(3)
            .map(|pos| {
                let hf: HashableFloat3 = pos[0..3].into();
                let n = vertex_to_avg_normal.get(&hf).unwrap();
                [n.x, n.y, n.z]
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    fn flat_shade(positions: &[f32]) -> Vec<f32> {
        positions
            .chunks_exact(9)
            .map(|verts| {
                let (v0, v1, v2) = (
                    make_vec3(&verts[0..3]),
                    make_vec3(&verts[3..6]),
                    make_vec3(&verts[6..9]),
                );

                let v0v1 = v1 - v0;
                let v0v2 = v2 - v0;

                let n: Vec3 = normalize(&cross(&v0v1, &v0v2));

                [n.x, n.y, n.z, n.x, n.y, n.z, n.x, n.y, n.z]
            })
            .flatten()
            .collect::<Vec<f32>>()
    }

    pub fn extract_attribute_indices<T, F: Fn(&IndexTuple) -> T>(
        obj_group: &Group,
        extractor: F,
    ) -> Vec<T> {
        obj_group
            .polys
            .iter()
            .map(|poly| poly.0.iter().map(|poly| extractor(poly)))
            .flatten()
            .collect::<Vec<_>>()
    }
}
