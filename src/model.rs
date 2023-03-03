use std::collections::HashSet;

use glow::HasContext;
use obj::{Obj, ObjError, MtlLibsLoadError};

use crate::loaders::*;

use crate::{
    egui_drawable::EguiDrawable,
    texture::{Texture2D, TextureType},
    gl::get_gl,
    mesh::Mesh,
    transform::Transform, 
    camera::Camera
};


#[derive(Default, Copy, Clone, Debug)]
pub enum ModelType {
    #[default]
    Normal,
    Billboard
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub shader_name: Option<String>,
    pub directory: String,
    pub model_type: ModelType
}

impl Model {
    pub fn load(path: impl Into<String>, texture_loader: &mut TextureLoader) -> Result<Model, ObjLoadError> {
        let loaded_model = Model::load_obj(path, texture_loader)?;

        // loaded_model.with_shader_name("default");
        // let default_texture = texture_loader.borrow("default");
        // loaded_model.add_texture(&Texture2D::from_handle(&default_texture, TextureType::Diffuse));

        Ok(loaded_model)
    }

    pub fn with_shader_name(&mut self, shader_name: &str) -> &mut Self {
        self.shader_name = Some(String::from(shader_name));
        self
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    #[allow(dead_code)]
    pub fn get_mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
    }

    pub fn draw_normal(&self, shader_loader: &mut ShaderLoader, transform: &Transform) {
        let gl_rc = get_gl();

        let def = DEFAULT_SHADER.to_string();
        let shader_name = self.shader_name.as_ref().unwrap_or(&def);

        let shader = shader_loader.borrow_shader(&shader_name).unwrap();

        unsafe { gl_rc.use_program(Some(shader.handle)); };

        shader.set_mat4("model", transform.get_model_matrix());

        for mesh in &self.meshes {
            mesh.draw(shader, "u_material.");
        }
    }

    pub fn draw_billboard(&self, shader_loader: &mut ShaderLoader, transform: &mut Transform, camera: &Camera) {
        let shader_name = self.shader_name.as_ref().unwrap();
        let shader = shader_loader.borrow_shader(shader_name).unwrap();

        shader.use_program();

        shader.set_mat4("view", camera.get_view_matrix());
        shader.set_mat4("projection", camera.get_proj_matrix());
        shader.set_mat4("model", transform.get_model_matrix());
        shader.set_vec3("billboard_center", *transform.get_pos());
        shader.set_float("billboard_size", 0.1);

        for mesh in &self.meshes {
            mesh.draw(shader, "");
        }
    }

    pub fn add_texture(&mut self, texture: &Texture2D) -> &mut Self {
        for mesh in &mut self.meshes {
            mesh.add_texture(texture);
        }
        self
    }
}

impl EguiDrawable for Model {
    #[allow(unused_variables)]
    fn on_egui(&mut self, ui: &mut egui::Ui, index: usize) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum ObjLoadError {
    Obj(ObjError),
    Mtl(MtlLibsLoadError),
}

impl From<ObjError> for ObjLoadError {
    fn from(value: ObjError) -> Self {
        ObjLoadError::Obj(value)
    }
}

impl From<MtlLibsLoadError> for ObjLoadError {
    fn from(value: MtlLibsLoadError) -> Self {
        ObjLoadError::Mtl(value)
    }
}

impl Model {
    pub fn load_obj(
        path: impl Into<String>,
        texture_loader: &mut TextureLoader,
    ) -> Result<Model, ObjLoadError> {
        
        let mut objects = Obj::load(path.into())?;

        // Must run this for materials to properly load
        objects.load_mtls()?;

        let dir = objects.path;

        let all_pos = objects.data.position;
        let all_norm = objects.data.normal;
        let all_tex = objects.data.texture;

        let mut model = Model {
            meshes: Vec::new(),
            directory: String::from(dir.to_str().unwrap()),
            shader_name: None,
            model_type: ModelType::Normal
        };

        for (object_index, object) in objects.data.objects.iter().enumerate()  {
            let num_objects = objects.data.objects.len() as f32;
            let progress_percentage = (object_index + 1) as f32 / num_objects;
            println!("Loading object {}%", progress_percentage * 100.0);

            let obj_group = &object.groups[0];

            let mut pnt: Vec<f32> = Vec::new();
            let mut inds: Vec<u32> = Vec::new();
            let mut index = 0u32;
            let mut textures: HashSet<Texture2D> = HashSet::new();

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

                inds.extend(vec![index as u32, (index + 1) as u32, (index + 2) as u32]);
                index += 3;

                if let Some(obj_mat) = &obj_group.material {
                    match obj_mat {
                        obj::ObjMaterial::Ref(_) => todo!(),
                        obj::ObjMaterial::Mtl(material) => {
                            if let Some(diffuse_map) = &material.map_kd {
                                let tex_handle = texture_loader
                                    .load_texture(&dir.join(diffuse_map));

                                let texture =
                                    Texture2D::from_native_handle(tex_handle, TextureType::Diffuse);

                                if !textures.contains(&texture) {
                                    textures.insert(texture);
                                }
                            }

                            if let Some(spec_map) = &material.map_ks {
                                let tex_handle = texture_loader
                                    .load_texture(&dir.join(spec_map));

                                let texture =
                                    Texture2D::from_native_handle(tex_handle, TextureType::Specular);

                                if !textures.contains(&texture) {
                                    textures.insert(texture);
                                }
                            }
                        }
                    }
                }
            }

            model.add_mesh(Mesh::new(
                &pnt,
                &inds,
                textures.iter().cloned().collect()
            ));
        }

        Ok(model)
    }
}
