use std::{mem::size_of, rc::Rc};

use byteorder::{LittleEndian, WriteBytesExt};
use glow::HasContext;
use image::EncodableLayout;

use crate::{gl::get_gl, shader::{UniformMap}, renderer::{Material, MaterialType}, loaders::ShaderLoader};
use std::hash::{Hasher, Hash};

#[derive(Hash)]
#[derive(Clone)]
pub struct MeshRenderer(pub Rc<Mesh>, pub Material);

impl MeshRenderer {
    pub fn new(mesh: Rc<Mesh>, material: Material) -> Self {
        Self ( mesh, material )
    }

    pub fn draw(&self, uniforms: &UniformMap) {
        let MeshRenderer(mesh, material) = self;

        material.shader.use_program();
        material.upload_uniforms(uniforms, "");

        let prefix = match material.material_type {
            MaterialType::Lit          => "u_material.",
            _                          => ""
        };

        material.upload_textures(prefix);

        unsafe {
            let gl_rc = get_gl();
            gl_rc.bind_vertex_array(Some(mesh.vao));
            gl_rc.draw_elements(
                glow::TRIANGLES,
                mesh.ind_data.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }

    pub fn set_material(&mut self, mat: Material) {
        self.1 = mat;
    }

    pub fn is_transparent(&self) -> bool { self.1.transparent }
}

#[derive(Clone)]
pub struct Mesh {
    // Buffer containing 3 floats for position, 3 for vertex normals, 2 for texture coordinates
    pub vert_data: Vec<f32>,
    pub ind_data: Vec<u32>,

    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
}

impl Hash for Mesh {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vao.hash(state);
        self.vbo.hash(state);
        self.ebo.hash(state);
    }
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
        vert_data: Vec<f32>,
        ind_data: Vec<u32>
    ) -> Self {
        let gl_rc = get_gl();
        let mut mesh = Mesh {
            vert_data,
            ind_data,

            vao: unsafe { gl_rc.create_vertex_array().unwrap() },
            vbo: unsafe { gl_rc.create_buffer().unwrap() },
            ebo: unsafe { gl_rc.create_buffer().unwrap() },
        };

        mesh.setup_mesh();
        mesh
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

