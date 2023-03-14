

use obj::{Obj, ObjError, MtlLibsLoadError};

use crate::loaders::*;

use crate::renderer::{Material, MaterialType, RenderCommand};
use crate::{
    egui_drawable::EguiDrawable,
    texture::{Texture2D, TextureType},
    mesh::Mesh,
};


#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub directory: String,
}

impl Model {
    pub fn load(path: impl Into<String>, texture_loader: &mut TextureLoader, shader_loader: &mut ShaderLoader) -> Result<Model, ObjLoadError> {
        let loaded_model = Model::load_obj(path, texture_loader, shader_loader)?;

        // loaded_model.with_shader_name("default");
        // let default_texture = texture_loader.borrow("default");
        // loaded_model.add_texture(&Texture2D::from_handle(&default_texture, TextureType::Diffuse));

        Ok(loaded_model)
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    #[allow(dead_code)]
    pub fn get_mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
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
        _shader_loader: &mut ShaderLoader
    ) -> Result<Model, ObjLoadError> {
        
        let mut objects = Obj::load(path.into())?;

        // Must run this for materials to properly load
        objects.load_mtls().unwrap();

        let dir = objects.path;

        let all_pos = objects.data.position;
        let all_norm = objects.data.normal;
        let all_tex = objects.data.texture;

        let mut model = Model {
            meshes: Vec::new(),
            directory: String::from(dir.to_str().unwrap()),
        };

        for (object_index, object) in objects.data.objects.iter().enumerate()  {
            let num_objects = objects.data.objects.len() as f32;
            let progress_percentage = (object_index + 1) as f32 / num_objects;
            println!("Loading object {}%", progress_percentage * 100.0);

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

            let mat = Material {
                shader_ref: DEFAULT_LIT_SHADER,
                material_type: MaterialType::Lit,
                textures,
                transparent: false
            };

            model.add_mesh(Mesh::new(
                pnt,
                inds,
                mat
            ));
            
        }

        Ok(model)
    }
}
