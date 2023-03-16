use std::{
    sync::Arc,
    collections::HashMap,
    mem::size_of
};

use glow::HasContext;
use glutin::dpi::PhysicalSize;
use image::EncodableLayout;
use nalgebra_glm::{Vec3, vec3};

use crate::{
    gl::{set_gl, get_gl},
    app::EventLoop,
    camera::Camera,
    loaders::{*, utils::Handle},
    ecs::Ecs,
    light_system,
    transform::Transform,
    model::Model, 
    shader::Uniform, 
    map, mesh::{MeshRenderer}
};

use super::material::*;

pub type GlutinWindow = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;


struct Line {
    // start_x, start_y, start_z, end_x, end_y, end_z
    ends: [f32; 6],
    material: Material,

    vao: glow::VertexArray,
    vbo: glow::Buffer
}

impl Line {
    // pub fn new(shader_loader: &mut ShaderLoader) -> Self {
    // }
    pub fn upload_data(&self) {
        let gl = get_gl();

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                self.ends.as_bytes(),
                glow::STATIC_DRAW,
            );
        }
    }

    fn set_ends(&mut self, start: Vec3, end: Vec3) {
        self.ends = [
            start.x, start.y, start.z,
            end.x, end.y, end.z
        ];
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
    pub fn new(window: GlutinWindow, window_width: usize, window_height: usize, shader_loader: &mut ShaderLoader) -> Self {
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
            debug_line_vbo: vbo
            // debug_line: Line::new(shader_loader)
        }
    }

    pub fn window_resized(&self, physical_size: &PhysicalSize<u32>) {
        self.window.resize(*physical_size);
        unsafe { get_gl().viewport(0, 0, self.window_width, self.window_height); }
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        ecs: &mut Ecs,
        shader_loader: &mut ShaderLoader,
        time: f32
    ) {
        unsafe {
            let gl = get_gl();
            gl.enable(glow::DEPTH_TEST);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::BLEND);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        light_system(ecs, shader_loader, self, camera);
          
        let render_commands = Self::collect_render_commands(ecs);

        let (transparent, opaque): 
            (Vec<_>, Vec<_>) = render_commands.into_iter().partition(|rc| rc.1.is_transparent());

        opaque.iter().for_each(|opaque_rc| {
            let RenderCommand(t, mesh) = opaque_rc;
            Self::draw_mesh(t, camera, mesh);
        });

        Self::draw_transparent(transparent, &camera);

        // let end = vec3(time.sin() * 10., 1., time.cos() * 10.);
        // self.draw_line(vec3(0., 1., 0.), end);
    }

    pub fn draw_mesh(transform: &Transform, camera: &Camera, mr: &MeshRenderer) {
        let lit_uniforms = map! {
            "projection" => Uniform::Mat4(camera.get_proj_matrix()),
            "view"       => Uniform::Mat4(camera.get_view_matrix()),
            "model"      => Uniform::Mat4(transform.get_model_matrix()),
            "u_view_pos" => Uniform::Vec3(camera.get_pos()),
            "u_material.shininess" => Uniform::Float(32.0)
        };

        let other_uniforms = map! {
            "projection" => Uniform::Mat4(camera.get_proj_matrix()),
            "view"       => Uniform::Mat4(camera.get_view_matrix()),
            "model"      => Uniform::Mat4(transform.get_model_matrix())
        };

        let uniforms = match &mr.1.material_type {
            MaterialType::Lit => &lit_uniforms,
            MaterialType::Billboard | MaterialType::Unlit => &other_uniforms,
        };

        mr.draw(uniforms);
    }

    fn collect_render_commands(ecs: &mut Ecs) -> Vec<RenderCommand> {
        let rcs: Vec<Option<Vec<_>>> = 
            ecs.do_all(|model_handle: &Handle<Model>, transform: &Transform| {
            let model_handle = model_handle.borrow(); if !model_handle.enabled { return None; }
            Some(
                model_handle.mesh_renderers.iter()
                .map(|mr| { RenderCommand(transform.clone(), mr.clone()) })
                .collect::<Vec<_>>()
            )
        });

        rcs
            .into_iter()
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .flatten()
            .collect::<Vec<RenderCommand>>()
    }

    fn draw_transparent(mut transparent: Vec<RenderCommand>, camera: &Camera) {
        transparent.sort_by(|tr1, tr2| {
            let dist1 = (tr1.0.get_pos() - camera.get_pos()).len();
            let dist2 = (tr2.0.get_pos() - camera.get_pos()).len();

            dist1.cmp(&dist2).reverse()
        });

        transparent.iter().for_each(|tr_rc| {
            let RenderCommand(t, mesh) = tr_rc;
            Self::draw_mesh(t, camera, mesh);
        });
    }

    pub fn draw_line(&mut self, start: Vec3, end: Vec3) {

        let gl = get_gl();


        let ends = [
            start.x, start.y, start.z,
            end.x, end.y, end.z 
        ];

        unsafe {
            gl.bind_vertex_array(Some(self.debug_line_vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.debug_line_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                ends.as_bytes(),
                glow::STATIC_DRAW,
            );

            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                3 * size_of::<f32>() as i32,
                0
            );

            gl.enable_vertex_attrib_array(0);

            gl.draw_arrays(glow::LINES, 0, 2);
        };
        //         unsigned int buffer; // The ID, kind of a pointer for VRAM
        // glGenBuffers(1, &buffer); // Allocate memory for the triangle
        // glBindBuffer(GL_ARRAY_BUFFER, buffer); // Set the buffer as the active array
        // glBufferData(GL_ARRAY_BUFFER, 2 * sizeof(float), line, GL_STATIC_DRAW); // Fill the buffer with data
        // glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), 0); // Specify how the buffer is converted to vertices
        // glEnableVertexAttribArray(0); // Enable the vertex array}
    }

}

pub struct RenderCommand(Transform, MeshRenderer);


// OpenGL 
// TODO: Add outlining to objects.
// TODO: Proper transparency

// Plumbing
// TODO: Perhaps change UniformMap to an array of tuples?
// Also, add support for recursion so nested structs won't be as painful.
// Perhaps move the shader loader here?
