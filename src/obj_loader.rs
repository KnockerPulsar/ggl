extern crate itertools;
extern crate obj;

extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

use glow::{Context, HasContext};
use image::EncodableLayout;
use std::{mem::size_of, rc::Rc};

use crate::{
    asset_loader::TextureLoader,
    shader::ShaderProgram,
    texture::{Texture2D, TextureType},
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
        gl_rc: &Rc<Context>,
        vertices: &Vec<f32>,
        indices: &Vec<u32>,
        textures: &Vec<Texture2D>,
    ) -> Self {
        let mut mesh = Mesh {
            vert_data: vertices.clone(),
            ind_data: indices.clone(),
            textures: textures.clone(),
            vao: unsafe { gl_rc.create_vertex_array().unwrap() },
            vbo: unsafe { gl_rc.create_buffer().unwrap() },
            ebo: unsafe { gl_rc.create_buffer().unwrap() },
        };

        mesh.setup_mesh(gl_rc);

        mesh
    }

    pub fn draw(&self, gl_rc: &Rc<Context>, shader: &ShaderProgram) {
        let mut diffuse_num = 1u32;
        let mut specular_num = 1u32;

        unsafe {
            for i in 0..(self.textures.len() as u32) {
                gl_rc.active_texture(glow::TEXTURE0 + i);

                let name;
                let texture_number;
                match self.textures[i as usize].texture_type {
                    TextureType::Diffuse(_) => {
                        texture_number = diffuse_num;
                        diffuse_num += 1;
                        name = "texture_diffuse";
                    }
                    TextureType::Specular(_) => {
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

                shader.set_int(
                    gl_rc,
                    &format!("u_material.{}{}", name, texture_number),
                    i as i32,
                );
                gl_rc.bind_texture(glow::TEXTURE_2D, Some(self.textures[i as usize].handle));
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

    fn setup_mesh(&mut self, gl_rc: &Rc<Context>) {
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

            PNTVertex::setup_attribs(gl_rc, &self.vao);
        }
    }

    pub fn add_texture(&mut self, texture: &Texture2D) {
        let existing_pos = self.textures.iter().position(|tex| tex.handle == texture.handle);

        if existing_pos.is_none() {
            self.textures.push(texture.clone());
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub directory: String,
}

impl Model {
    pub fn load_model(gl_rc: &Rc<Context>, path: &str, texture_loader: &mut TextureLoader) -> Self {
        ObjLoader::load_obj(gl_rc, path, texture_loader)
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn get_mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
    }

    pub fn draw(&self, gl_rc: &Rc<Context>, shader: &ShaderProgram) {
        unsafe { gl_rc.use_program(Some(shader.handle)) };
        for mesh in &self.meshes {
            mesh.draw(gl_rc, shader);
        }
    }

    pub fn add_texture(&mut self, texture: &Texture2D) {
        for mesh in &mut self.meshes {
            mesh.add_texture(texture);
        }
    }
}

impl ObjLoader {
    pub fn load_obj(gl_rc: &Rc<Context>, path: &str, texture_loader: &mut TextureLoader) -> Model {
        let mut objects = Obj::load(path).unwrap();
        let _ = objects.load_mtls().unwrap();
        let dir = objects.path;

        let all_pos = objects.data.position;
        let all_norm = objects.data.normal;
        let all_tex = objects.data.texture;

        let mut model = Model {
            meshes: Vec::new(),
            directory: String::from(dir.to_str().unwrap()),
        };

        for object in objects.data.objects {
            let obj_group = &object.groups[0];

            let mut pnt: Vec<f32> = Vec::new();
            let mut inds: Vec<u32> = Vec::new();
            let mut index = 0u32;
            let mut textures: Vec<Texture2D> = Vec::new();

            let mut diff_tex_index = 1u32;
            let mut spec_tex_index = 1u32;
            for poly in obj_group.polys.iter() {
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
                                let (first_load, texture) = texture_loader.load_texture(
                                    gl_rc,
                                    dir.join(diffuse_map).to_str().unwrap(),
                                    TextureType::Diffuse(diff_tex_index),
                                );

                                if first_load {
                                    textures.push(texture.clone());

                                    diff_tex_index += 1;
                                }
                            }

                            if let Some(spec_map) = &material.map_ks {
                                let (first_load, texture) = texture_loader.load_texture(
                                    gl_rc,
                                    dir.join(spec_map).to_str().unwrap(),
                                    TextureType::Specular(spec_tex_index),
                                );

                                if first_load {
                                    textures.push(texture.clone());
                                    spec_tex_index += 1;
                                }
                            }
                        }
                    }
                }
            }

            model.add_mesh(Mesh::new(gl_rc, &pnt, &inds, &textures));
        }

        model
    }
}

pub trait VertexAttribs {
    fn setup_attribs(gl_rc: &Rc<Context>, vao: &glow::VertexArray);
}

// 3 floats for position, 3 for vertex normals, 2 for texture coordinates
#[derive(Clone)]
pub struct PNTVertex;

impl VertexAttribs for PNTVertex {
    fn setup_attribs(gl_rc: &Rc<Context>, vao: &glow::VertexArray) {
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
    fn setup_attribs(gl_rc: &Rc<Context>, vao: &glow::VertexArray) {
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
