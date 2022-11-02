extern crate itertools;
extern crate obj;

extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

use glow::HasContext;
use image::EncodableLayout;
use std::{collections::HashSet, mem::size_of};

use crate::{
    asset_loader::TextureLoader,
    egui_drawable::EguiDrawable,
    shader::ShaderProgram,
    shader_loader::ShaderLoader,
    texture::{Texture2D, TextureType}, get_gl,
};

use obj::Obj;

pub struct ObjLoader;

pub struct Mesh {
    // Buffer containing 3 floats for position, 3 for vertex normals, 2 for texture coordinates
    pub vert_data: Vec<f32>,
    pub ind_data: Vec<u32>,
    pub textures: Vec<Texture2D>,

    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
}

fn to_bytes(vu32: &mut Vec<u32>) -> Vec<u8> {
    let mut v8: Vec<u8> = Vec::new();

    for n in vu32 {
        v8.write_u32::<LittleEndian>(*n).unwrap();
    }

    v8
}

impl Mesh {
    pub fn new(
        vertices: &Vec<f32>,
        indices: &Vec<u32>,
        textures: Vec<Texture2D>,
    ) -> Self {
        let gl_rc = get_gl();
        let mut mesh = Mesh {
            vert_data: vertices.to_owned(),
            ind_data: indices.to_owned(),
            textures,
            vao: unsafe { gl_rc.create_vertex_array().unwrap() },
            vbo: unsafe { gl_rc.create_buffer().unwrap() },
            ebo: unsafe { gl_rc.create_buffer().unwrap() },
        };

        mesh.setup_mesh();

        mesh
    }

    pub fn draw(&self, shader: &ShaderProgram) {
        let mut diffuse_num = 1u32;
        let mut specular_num = 1u32;

        unsafe {
            let gl_rc = get_gl();
            for i in 0..(self.textures.len() as u32) {
                gl_rc.active_texture(glow::TEXTURE0 + i);

                let name;
                let texture_number;
                match self.textures[i as usize].tex_type {
                    TextureType::Diffuse => {
                        texture_number = diffuse_num;
                        diffuse_num += 1;
                        name = "texture_diffuse";
                    }
                    TextureType::Specular => {
                        texture_number = specular_num;
                        specular_num += 1;
                        name = "texture_specular"
                    }
                    // !Only one emissive for now
                    TextureType::Emissive => {
                        texture_number = 1;
                        name = "texture_emissive"
                    }
                }

                // println!("{}{}", name, texture_number);
                gl_rc.bind_texture(glow::TEXTURE_2D, Some(self.textures[i as usize].handle));
                shader.set_int(
                    &format!("u_material.{}{}", name, texture_number),
                    i as i32,
                );
            }

            gl_rc.bind_vertex_array(Some(self.vao));
            gl_rc.draw_elements(
                glow::TRIANGLES,
                self.ind_data.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }

    fn setup_mesh(&mut self) {
        let gl_rc = get_gl();
        unsafe {
            gl_rc.bind_vertex_array(Some(self.vao));
            gl_rc.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            gl_rc.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                self.vert_data.as_bytes(),
                glow::STATIC_DRAW,
            );

            gl_rc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl_rc.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                to_bytes(&mut self.ind_data).as_slice(),
                glow::STATIC_DRAW,
            );

            PNTVertex::setup_attribs(&self.vao);
        }
    }

    pub fn add_texture(&mut self, texture: &Texture2D) {
        if !self.textures.contains(texture) {
            self.textures.push(texture.clone());
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub shader_name: Option<String>,
    pub directory: String,
}

impl Model {
    pub fn load_model(path: &str, texture_loader: &mut TextureLoader) -> Self {
        ObjLoader::load_obj(path, texture_loader)
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

    pub fn draw(&self, shader_loader: &mut ShaderLoader) {
        // ! TODO: Add a default shader

        let gl_rc = get_gl();

        let shader_name = self.shader_name.as_ref().unwrap();
        let shader = shader_loader.borrow_shader(shader_name).unwrap();

        unsafe { gl_rc.use_program(Some(shader.handle)) };

        for mesh in &self.meshes {
            mesh.draw(shader);
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

impl ObjLoader {
    pub fn load_obj(
        path: &str,
        texture_loader: &mut TextureLoader,
    ) -> Model {
        
        let mut objects = Obj::load(path).unwrap();

        // Must run this for materials to properly load
        objects.load_mtls().unwrap();

        let dir = objects.path;

        let all_pos = objects.data.position;
        let all_norm = objects.data.normal;
        let all_tex = objects.data.texture;

        let mut model = Model {
            meshes: Vec::new(),
            directory: String::from(dir.to_str().unwrap()),
            shader_name: None,
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
                                let (_, tex_handle) = texture_loader
                                    .load_texture(dir.join(diffuse_map).to_str().unwrap());

                                let texture =
                                    Texture2D::from_handle(tex_handle, TextureType::Diffuse);

                                if !textures.contains(&texture) {
                                    textures.insert(texture);
                                }
                            }

                            if let Some(spec_map) = &material.map_ks {
                                let (_, tex_handle) = texture_loader
                                    .load_texture(dir.join(spec_map).to_str().unwrap());

                                let texture =
                                    Texture2D::from_handle(tex_handle, TextureType::Specular);

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

        model
    }
}

pub trait VertexAttribs {
    fn setup_attribs(vao: &glow::VertexArray);
}

// 3 floats for position, 3 for vertex normals, 2 for texture coordinates
#[derive(Clone)]
pub struct PNTVertex;

impl VertexAttribs for PNTVertex {
    fn setup_attribs(vao: &glow::VertexArray) {
        let gl_rc = get_gl();

        unsafe {
            gl_rc.bind_vertex_array(Some(*vao));

            gl_rc.enable_vertex_attrib_array(0);
            gl_rc.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                8 * size_of::<f32>() as i32,
                0,
            );

            gl_rc.enable_vertex_attrib_array(1);
            gl_rc.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                8 * size_of::<f32>() as i32,
                3 * size_of::<f32>() as i32,
            );

            gl_rc.enable_vertex_attrib_array(2);
            gl_rc.vertex_attrib_pointer_f32(
                2,
                2,
                glow::FLOAT,
                false,
                8 * size_of::<f32>() as i32,
                6 * size_of::<f32>() as i32,
            );
        }
    }
}

pub struct PVertex;
impl VertexAttribs for PVertex {
    fn setup_attribs(vao: &glow::VertexArray) {
        let gl_rc = get_gl();

        unsafe {
            gl_rc.bind_vertex_array(Some(*vao));

            gl_rc.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                8 * size_of::<f32>() as i32,
                0,
            );
            gl_rc.enable_vertex_attrib_array(0);
        }
    }
}
