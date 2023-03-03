#![allow(dead_code)]

extern crate itertools;
extern crate obj;
extern crate byteorder;

use std::collections::HashMap;

use crate::{
    texture::{Texture2D, TextureType},
    model::{Model, ModelType, ObjLoadError}, 
    mesh::Mesh, egui_drawable::EguiDrawable,
    loaders::*, enabled_header
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
            pub fn $fn_name(&self) -> &Model {
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
}

impl ObjLoader {
    fn load_default_cube(&mut self, _shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader){
        let cube_path = "assets/obj/cube.obj";

        let mut cube_model = Model::load_obj(cube_path, texture_loader).unwrap();
        let checker_texture = texture_loader.checker_texture();

        cube_model.add_texture(&Texture2D { 
            native_handle: checker_texture, 
            tex_type: TextureType::Diffuse
        }).add_texture(&Texture2D { 
            native_handle: checker_texture,
            tex_type: TextureType::Specular
        }).with_shader_name(DEFAULT_SHADER);

        self.models.insert(DEFAULT_CUBE_NAME.into(), cube_model);
    }

    fn load_default_plane(&mut self, _shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) {
        
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
        
        let textures: Vec<Texture2D> = vec![
            Texture2D::from_native_handle(texture_loader.point_light_texture(), TextureType::Diffuse)
        ];
        
        let mesh = Mesh::new(&vertices, &indices, textures);
        
        let default_square = Model {
            meshes: vec![mesh],
            shader_name: Some(DEFAULT_BILLBOARD_SHADER.to_string()),
            directory: "".to_string(),
            model_type: ModelType::Billboard
        };

        self.models.insert(DEFAULT_PLANE_NAME.into(), default_square);
    }

    pub fn new(shader_loader: &mut ShaderLoader, texture_loader: &mut TextureLoader) -> Self {
        let mut obj_loader = ObjLoader { models: HashMap::new() };

        obj_loader.load_default_cube(shader_loader, texture_loader);
        obj_loader.load_default_plane(shader_loader, texture_loader);

        obj_loader
    }

    pub fn load(&mut self, path: impl Into<String>, texture_loader: &mut TextureLoader) -> ModelLoadResult<()> {
        let path_string: String = path.into();

        if !self.models.contains_key(&path_string) {
            self.models.insert(path_string.clone(), Model::load(&path_string, texture_loader)?);
        }

        self.models.get_mut(&path_string).unwrap(); // Should always succeed
        Ok(())
    }

    pub fn borrow(&self, model_path: &str) -> &Model {
        match self.models.get(model_path) {
            Some(model) => model,
            None => self.default_cube_model(),
        }
    }

    pub fn models(&mut self) -> &mut HashMap<String, Model> {
        &mut self.models
    }

    default_model_getters![
        (DEFAULT_CUBE_NAME, default_cube_model),
        (DEFAULT_PLANE_NAME, default_plane_model)
    ];
}

