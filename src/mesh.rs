use std::mem::size_of;

use byteorder::{LittleEndian, WriteBytesExt};
use glow::HasContext;
use image::EncodableLayout;

use crate::gl::{get_gl, GlContext};
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct Mesh {
    // Buffer containing 3 floats for position, 3 for vertex normals, 2 for texture coordinates
    pub vertex_positions: Vec<f32>,
    pub vertex_normals: Vec<f32>,
    pub vertex_texture_coordinates: Vec<f32>,

    pub indices: Vec<u32>,

    vao: glow::VertexArray,

    position_bo: glow::Buffer,
    normal_bo: Option<glow::Buffer>,
    texture_coordinate_bo: Option<glow::Buffer>,

    ebo: glow::Buffer,
}

impl Hash for Mesh {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vao.hash(state);
        self.position_bo.hash(state);
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
        positions: Vec<f32>,
        normals: Vec<f32>,
        texture_coordinates: Vec<f32>,
        indices: Vec<u32>,
    ) -> Self {
        let gl_rc = get_gl();

        let normal_bo = if !normals.is_empty() {
            Some(unsafe { gl_rc.create_buffer().unwrap() })
        } else {
            None
        };

        let texture_coordinate_bo = if !texture_coordinates.is_empty() {
            Some(unsafe { gl_rc.create_buffer().unwrap() })
        } else {
            None
        };

        let mut mesh = Mesh {
            vertex_positions: positions,
            vertex_normals: normals,
            vertex_texture_coordinates: texture_coordinates,
            indices,

            vao: unsafe { gl_rc.create_vertex_array().unwrap() },
            position_bo: unsafe { gl_rc.create_buffer().unwrap() },
            normal_bo,
            texture_coordinate_bo,
            ebo: unsafe { gl_rc.create_buffer().unwrap() },
        };

        mesh.setup();
        mesh
    }

    fn setup(&mut self) {
        let gl_ctx = get_gl();
        unsafe {
            gl_ctx.bind_vertex_array(Some(self.vao));

            {
                gl_ctx.bind_buffer(glow::ARRAY_BUFFER, Some(self.position_bo));
                gl_ctx.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    self.vertex_positions.as_bytes(),
                    glow::STATIC_DRAW,
                );

                position_attribs(gl_ctx);
            }

            if let Some(normal_bo) = &self.normal_bo {
                gl_ctx.bind_buffer(glow::ARRAY_BUFFER, Some(*normal_bo));
                gl_ctx.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    self.vertex_normals.as_bytes(),
                    glow::STATIC_DRAW,
                );

                normal_attribs(gl_ctx);
            };

            if let Some(texture_coordinate_bo) = &self.texture_coordinate_bo {
                gl_ctx.bind_buffer(glow::ARRAY_BUFFER, Some(*texture_coordinate_bo));
                gl_ctx.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    self.vertex_texture_coordinates.as_bytes(),
                    glow::STATIC_DRAW,
                );

                texture_coordinate_attribs(gl_ctx)
            };

            gl_ctx.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl_ctx.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                to_bytes(&mut self.indices).as_slice(),
                glow::STATIC_DRAW,
            );
        }
    }

    pub fn vao(&self) -> glow::VertexArray {
        self.vao
    }
}

fn position_attribs(gl_ctx: &GlContext) {
    unsafe {
        gl_ctx.enable_vertex_attrib_array(0);
        gl_ctx.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);
    }
}

fn normal_attribs(gl_ctx: &GlContext) {
    unsafe {
        gl_ctx.enable_vertex_attrib_array(1);
        gl_ctx.vertex_attrib_pointer_f32(
            1,
            3,
            glow::FLOAT,
            false,
            3 * size_of::<f32>() as i32,
            0, // Using a separate buffer for normals.
        );
    }
}

fn texture_coordinate_attribs(gl_ctx: &GlContext) {
    unsafe {
        gl_ctx.enable_vertex_attrib_array(2);
        gl_ctx.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 2 * size_of::<f32>() as i32, 0);
    }
}
