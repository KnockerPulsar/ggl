#![allow(dead_code)]

extern crate itertools;
extern crate obj;
extern crate byteorder;

use std::{collections::HashMap, rc::Rc};

use obj::Obj;

use crate::{
    texture::{Texture2D, TextureType},
    model::{Model, ObjLoadError}, 
    mesh::{Mesh, MeshRenderer},
    loaders::*, renderer::{Material, MaterialType}, map
};

use self::utils::Handle;

pub mod utils {
    use std::{ops::{Deref, DerefMut}, rc::Rc, cell::RefCell};

    use crate::egui_drawable::EguiDrawable;

    pub struct Handle<T>(Rc<RefCell<T>>);

    impl<T> Handle<T> {
        pub fn new(t: T) -> Self {
            Handle(Rc::new(RefCell::new(t)))
        }
    }

    impl<T> Deref for Handle<T> {
        type Target=RefCell<T>;

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
        fn on_egui(&mut self, ui: &mut egui::Ui, index: usize) -> bool {
            self.0.deref().borrow_mut().deref_mut().on_egui(ui, index)
        }
    }
}


pub type ModelLoadResult<T> = std::result::Result<T, ObjLoadError>;

pub const DEFAULT_CUBE_NAME: &str = "default_cube";
pub const DEFAULT_PLANE_NAME: &str = "default_plane";


pub struct ObjLoader {
    models: HashMap<String, Handle<Model>>,
    model_meshes: HashMap<String, Vec<Rc<Mesh>>>
}

impl ObjLoader {
    fn load_default_cube(&mut self, shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) {
        let cube_path = "assets/obj/cube.obj";

        let cube_model = self.load_obj(DEFAULT_CUBE_NAME, cube_path, texture_loader, shader_loader).unwrap();
        let mat = Material::default_lit(shader_loader, texture_loader);

        for mr in &mut cube_model.borrow_mut().mesh_renderers {
            mr.set_material(mat.clone());
        }
    }

    fn load_default_plane(&mut self, shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) {
        
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
        let vertices: Vec<f32> = vec![
            -0.5, 0.5, 0.,  // Position
            0., 0., 1.,     // Normal
            0., 0.,         // UVs
            
            0.5, 0.5, 0.,
            0., 0., 1., 
            1., 0.,    
                           
            -0.5, -0.5, 0.,
            0., 0., 1.,
            0., 1.,    

            0.5, -0.5, 0.,
            0., 0., 1.,
            1., 1.,    
        ];

        let indices: Vec<u32> = vec![
            2, 0, 1,
            2, 1, 3
        ];
        

        let mat = Material::default_billboard(shader_loader, texture_loader);
        let t = self.add_mesh(DEFAULT_PLANE_NAME.into(), Mesh::new(vertices, indices));
        self.add_model(
            DEFAULT_PLANE_NAME, 
            Model::new(
                DEFAULT_PLANE_NAME, 
                "",
                vec![
                    MeshRenderer::new(
                        t, 
                        mat
                    )
                ]
            )
        );
    }

    pub fn new(shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) -> Self {
        let mut loader = ObjLoader {models: map! {}, model_meshes: map! {}};

        loader.load_default_cube(shader_loader, texture_loader);
        loader.load_default_plane(shader_loader, texture_loader);

        loader
    }

    pub fn add_mesh(&mut self, model_key: String, mesh: Mesh) -> Rc<Mesh> {
        self
            .model_meshes
            .entry(model_key.clone())
            .or_insert(vec![])
            .push(Rc::new(mesh));

        let r = self.model_meshes.get(&model_key).unwrap().last().unwrap();
        Rc::clone(r)
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
        texture_loader: &mut TextureLoader,
        shader_loader: &mut ShaderLoader
    ) -> ModelLoadResult<Handle<Model>> {
        let path_string: String = path.into();
        let name: String = name.into();

        if !self.models.contains_key(&path_string) {
            self.load_obj(name.clone(), &path_string, texture_loader, shader_loader)?;
        }

        let t = self.models.get(&name).unwrap();
        Ok(Handle::clone(t)) // Should always succeed
    }

    pub fn clone_handle(&mut self, model_key: &str) -> Handle<Model> {
        Handle::clone(
            self
            .models
            .get(model_key)
            .unwrap()
        )
    }
    
    pub fn clone(&mut self, old_name: &str, new_name: &str) -> Handle<Model> {
        assert!(new_name != old_name, "New and old model names must be different!");

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
        texture_loader: &mut TextureLoader,
        shader_loader: &mut ShaderLoader,
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
            Model::new(dir.clone(), dir, Vec::new())
        };
        let num_objects = objects.data.objects.len() as f32;

        for (object_index, object) in objects.data.objects.iter().enumerate()  {
            println!("Loading object {i}/{num_objects}", i = object_index);

            let obj_group = &object.groups[0];

            let mut pnt: Vec<f32> = vec![];
            let mut inds: Vec<u32> = vec![];
            let mut index = 0u32;

            let mut textures: Vec<Texture2D> = vec![];
            let mut num_diffuse = 1;
            let mut num_specular = 1;
            let _num_emissive = 1;

            for (_, poly) in obj_group.polys.iter().enumerate() {
                for vertex in &poly.0 {
                    let pos_index = vertex.0;

                    pnt.extend(all_pos[pos_index]);

                    if let Some(norm_index) = vertex.2 {
                        pnt.extend(all_norm[norm_index]);
                    }

                    if let Some(tex_index) = vertex.1 {
                        pnt.extend(all_tex[tex_index]);
                    }
                }

                inds.extend(vec![index, (index + 1), (index + 2)]);
                index += 3;

                if let Some(obj_mat) = &obj_group.material {
                    match obj_mat {
                        obj::ObjMaterial::Ref(_) => todo!(),
                        obj::ObjMaterial::Mtl(material) => {
                            if let Some(diffuse_map) = &material.map_kd {
                                let tex_handle = texture_loader
                                    .load_texture(&dir.join(diffuse_map));

                                let texture =
                                    Texture2D::from_native_handle(
                                        tex_handle, 
                                        TextureType::Diffuse,
                                        num_diffuse
                                    );

                                if !textures.contains(&texture) {
                                    textures.push(texture);
                                    num_diffuse += 1;
                                }
                            }

                            if let Some(spec_map) = &material.map_ks {
                                let tex_handle = texture_loader
                                    .load_texture(&dir.join(spec_map));

                                let texture =
                                    Texture2D::from_native_handle(
                                        tex_handle, 
                                        TextureType::Specular,
                                        num_specular
                                    );

                                if !textures.contains(&texture) {
                                    textures.push(texture);
                                    num_specular += 1;
                                }
                            }
                        }
                    }
                }
            }

            let mat = Material::lit(shader_loader, textures);

            
            let mesh = self.add_mesh(name.clone(), Mesh::new(pnt, inds));
            model.add_mesh(MeshRenderer::new(mesh, mat));
        }

        Ok(self.add_model(name, model))
    }
}
