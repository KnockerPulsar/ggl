#![allow(dead_code)]

extern crate itertools;
extern crate obj;
extern crate byteorder;

use std::collections::HashMap;

use crate::{
    texture::{Texture2D, TextureType},
    model::{Model, ObjLoadError}, 
    mesh::Mesh, egui_drawable::EguiDrawable,
    loaders::*, enabled_header, renderer::Material, shader::ShaderProgram
};

#[derive(Debug, Clone, Default)]
pub struct ModelHandle {
    name: String,
    enabled: bool
}

impl From<String> for ModelHandle {
    fn from(name: String) -> Self {
        ModelHandle { name, enabled: true }
    }
}

impl From<&str> for ModelHandle {
    fn from(name: &str) -> Self {
        ModelHandle { name: name.into(), enabled: true }
    }
}

impl ModelHandle {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

pub type ModelLoadResult<T> = std::result::Result<T, ObjLoadError>;

pub const DEFAULT_CUBE_NAME: &'static str = "default_cube";
pub const DEFAULT_PLANE_NAME: &'static str = "default_plane";


macro_rules! default_model_getters {
    ($( ($name: expr, $fn_name: tt) ),*) => {
        $(
            #[allow(dead_code)]
            pub fn $fn_name(&mut self) -> &mut Model {
                self.borrow($name)
            }
        )*
    };
}


impl EguiDrawable for ModelHandle {
    fn on_egui(&mut self, ui: &mut egui::Ui, index: usize) -> bool {
        enabled_header!(self, ui, "Model", index, {
            ui.horizontal(|ui| {
                ui.label(format!("Name: {}", self.name));

                if ui.button("Load model").clicked() {
                    let path = rfd::FileDialog::new().add_filter("Object model", &["obj"]).pick_file(); 
                    if let Some(path) = path { 
                        let t = path.to_str().unwrap_or_else(|| DEFAULT_CUBE_NAME).to_owned();
                        self.name = t;
                    }
                }
            });
        });
        false
    }
}

pub struct ObjLoader {
    models: HashMap<String, Model>,
    default_cube: Model,
    default_plane: Model
}

impl ObjLoader {
    fn load_default_cube(shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) -> Model {
        let cube_path = "assets/obj/cube.obj";

        let mut cube_model = Model::load_obj(cube_path, texture_loader, shader_loader).unwrap();
        let checker_texture = texture_loader.checker_texture();

        cube_model
        // .add_texture(&Texture2D { 
        //     native_handle: checker_texture, 
        //     tex_type: TextureType::Diffuse,
        //     tex_index: 1,
        //     tex_unit: 0
        // })
        // .add_texture(&Texture2D { 
        //     native_handle: checker_texture,
        //     tex_type: TextureType::Specular,
        //     tex_index: 1,
        //     tex_unit: 1
        // })
        .with_material(Material::default_unlit(shader_loader));

        cube_model
    }

    fn load_default_plane(shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) -> Model {
        
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
        
        let default_texture = Texture2D::from_native_handle(
            texture_loader.directional_light_texture(), 
            TextureType::Diffuse, 1
        );

        let textures: Vec<Texture2D> = vec![
            default_texture.clone()
        ];
        
        let mesh = Mesh::new(vertices, indices, textures);
        
        let default_square = Model {
            meshes: vec![mesh],
            directory: "".to_string(),
            material: Material::default_billboard(texture_loader)
        };

        default_square
    }

    pub fn new(shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) -> Self {
        ObjLoader { 
            models: HashMap::new(),
            default_cube: Self::load_default_cube(shader_loader, texture_loader),
            default_plane: Self::load_default_plane(shader_loader, texture_loader)
        }
    }

    pub fn load(&mut self, path: impl Into<String>, texture_loader: &mut TextureLoader, shader_loader: &mut ShaderLoader) -> ModelLoadResult<()> {
        let path_string: String = path.into();

        if [DEFAULT_CUBE_NAME, DEFAULT_PLANE_NAME].contains(&path_string.as_str()) {
            return Ok(());
        }

        if !self.models.contains_key(&path_string) {
            self.models.insert(path_string.clone(), Model::load(&path_string, texture_loader, shader_loader)?);
        }

        self.models.get_mut(&path_string).unwrap(); // Should always succeed
        Ok(())
    }

    pub fn borrow(&mut self, model_path: &str) -> &mut Model {

        if model_path == DEFAULT_CUBE_NAME {
            return &mut self.default_cube
        } else if model_path == DEFAULT_PLANE_NAME {
            return &mut self.default_plane
        }

        self
            .models
            .get_mut(model_path)
            .unwrap_or(&mut self.default_cube)
    }
    
    pub fn clone(&mut self, old_name: &str, new_name: &str) -> &mut Model {
        assert!(new_name != old_name, "New and old model names must be different!");

        let new_model = self.borrow(old_name).clone();
        self.models.insert(new_name.to_string(), new_model);

        self.borrow(new_name)
    }

    pub fn models(&mut self) -> &mut HashMap<String, Model> {
        &mut self.models
    }

    default_model_getters![
        (DEFAULT_CUBE_NAME, default_cube_model),
        (DEFAULT_PLANE_NAME, default_plane_model)
    ];
}

