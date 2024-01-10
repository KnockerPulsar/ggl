use std::{collections::HashMap, mem::size_of, sync::Arc};

use glow::HasContext;
use glutin::dpi::PhysicalSize;
use image::EncodableLayout;
use nalgebra_glm::{vec3, Vec3};

use crate::{
    app::EventLoop,
    camera::Camera,
    ecs::Ecs,
    gl::{get_gl, set_gl},
    light_system,
    loaders::{utils::Handle, *},
    map,
    model::Model,
    shader::Uniform,
    transform::Transform,
};

use super::material::*;

pub type GlutinWindow = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

struct Line {
    // start_x, start_y, start_z, end_x, end_y, end_z
    ends: [f32; 6],
    material: Material,

    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl Line {
    // pub fn new(shader_loader: &mut ShaderLoader) -> Self {
    // }
    pub fn upload_data(&self) {
        let gl = get_gl();

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, self.ends.as_bytes(), glow::STATIC_DRAW);
        }
    }

    fn set_ends(&mut self, start: Vec3, end: Vec3) {
        self.ends = [start.x, start.y, start.z, end.x, end.y, end.z];
    }

    pub fn draw(&mut self, start: Vec3, end: Vec3) {
        let gl = get_gl();
        self.set_ends(start, end);
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            self.upload_data();

            gl.line_width(0.1);
            gl.draw_arrays(glow::LINES, 0, 2);
        }
    }
}

pub struct Renderer {
    pub window_width: i32,
    pub window_height: i32,
    pub window: GlutinWindow,
    pub lights_on: bool,

    // debug_line: Line
    debug_line_vao: glow::NativeVertexArray,
    debug_line_vbo: glow::NativeBuffer,
}

impl Renderer {
    pub fn new(
        window: GlutinWindow,
        window_width: usize,
        window_height: usize,
        shader_loader: &mut ShaderLoader,
    ) -> Self {
        let window_width = window_width as i32;
        let window_height = window_height as i32;

        let gl = get_gl();
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };

        Renderer {
            window_width,
            window_height,
            window,
            lights_on: true,

            debug_line_vao: vao,
            debug_line_vbo: vbo, // debug_line: Line::new(shader_loader)
        }
    }

    pub fn window_resized(&self, physical_size: &PhysicalSize<u32>) {
        self.window.resize(*physical_size);
        unsafe {
            get_gl().viewport(0, 0, self.window_width, self.window_height);
        }
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        ecs: &mut Ecs,
        shader_loader: &mut ShaderLoader,
        time: f32,
    ) {
        unsafe {
            let gl = get_gl();
            gl.enable(glow::DEPTH_TEST);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::BLEND);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        light_system(ecs, shader_loader, self);

        let render_commands = Self::collect_render_commands(ecs);

        let (transparent, opaque): (Vec<_>, Vec<_>) =
            render_commands
                .into_iter()
                .partition(|RenderCommand(_, model)| {
                    model.borrow().material.as_ref().unwrap().transparent
                });

        opaque.iter().for_each(|opaque_rc| {
            Self::draw_model_with_uniforms(opaque_rc, camera);
        });

        Self::draw_transparent(transparent, &camera);
    }

    pub fn draw_model_with_uniforms(rc: &RenderCommand, camera: &Camera) {
        let RenderCommand(transform, model) = rc;

        // TODO: Need some way to be able to:
        //  1. Bind the shader/material and set up per-pass uniforms once
        //  2. Allow the material to carry its own uniforms
        let uniforms = map! {
            "projection" => Uniform::Mat4(camera.get_proj_matrix()),
            "view"       => Uniform::Mat4(camera.get_view_matrix()),
            "u_view_pos" => Uniform::Vec3(camera.get_pos()),
            "model"      => Uniform::Mat4(transform.get_model_matrix())
        };

        model.borrow().draw(&uniforms);
    }

    fn collect_render_commands(ecs: &mut Ecs) -> Vec<RenderCommand> {
        ecs.query2::<Handle<Model>, Transform>()
            .unwrap()
            .iter()
            .filter_map(|(h, t)| Option::zip(h.as_ref(), t.as_ref()))
            .map(|(model_handle, transform)| {
                if !model_handle.borrow().enabled {
                    return None;
                }

                Some(RenderCommand(
                    transform.clone(),
                    Handle::clone(model_handle),
                ))
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>()
    }

    fn draw_transparent(mut transparent: Vec<RenderCommand>, camera: &Camera) {
        transparent.sort_by(|tr1, tr2| {
            let dist1 = (tr1.0.get_pos() - camera.get_pos()).len();
            let dist2 = (tr2.0.get_pos() - camera.get_pos()).len();

            dist1.cmp(&dist2).reverse()
        });

        transparent.iter().for_each(|tr_rc| {
            Self::draw_model_with_uniforms(tr_rc, camera);
        });
    }

    #[allow(dead_code)]
    pub fn draw_line(&mut self, start: Vec3, end: Vec3) {
        let gl = get_gl();

        let ends = [start.x, start.y, start.z, end.x, end.y, end.z];

        unsafe {
            gl.bind_vertex_array(Some(self.debug_line_vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.debug_line_vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, ends.as_bytes(), glow::STATIC_DRAW);

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);

            gl.enable_vertex_attrib_array(0);

            gl.draw_arrays(glow::LINES, 0, 2);
        };
    }
}

// TODO: Carry references to avoid copies?
pub struct RenderCommand(Transform, Handle<Model>);

// OpenGL
// TODO: Add outlining to objects.
// TODO: Proper transparency

// Plumbing
// TODO: Perhaps change UniformMap to an array of tuples?
// Also, add support for recursion so nested structs won't be as painful.
// Perhaps move the shader loader here?
