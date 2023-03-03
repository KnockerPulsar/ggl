use std::mem::size_of;

use byteorder::{LittleEndian, WriteBytesExt};
use glow::HasContext;
use image::EncodableLayout;

use crate::{texture::{Texture2D, TextureType}, gl::get_gl, shader::ShaderProgram};

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

    pub fn draw(&self, shader: &ShaderProgram, texture_parent: impl Into<String>) {
        let mut diffuse_num = 1u32;
        let mut specular_num = 1u32;

        unsafe {
            let gl_rc = get_gl();
            let texture_parent = texture_parent.into();

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

                // println!("{:?}", self.textures[i as usize]);
                gl_rc.bind_texture(glow::TEXTURE_2D, Some(self.textures[i as usize].native_handle));
                shader.set_int(
                    &format!("{}{}{}", texture_parent , name, texture_number),
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

