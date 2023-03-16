use std::{
    sync::Arc,
    collections::HashMap
};

use glow::HasContext;
use glutin::dpi::PhysicalSize;

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

pub struct Renderer {
    pub window_width: i32,
    pub window_height: i32,
    pub window: GlutinWindow,
    pub lights_on: bool,
}

impl Renderer {
    pub fn new(window_width: usize, window_height: usize, event_loop: &EventLoop) -> Self {

        let window_width = window_width as i32;
        let window_height = window_height as i32;
        let window = Self::init_window(window_width, window_height, event_loop);

        println!("Loading, please wait...");

        Renderer {
            window_width,
            window_height,
            window,
            lights_on: true,
        }
    }

    fn init_window(window_width: i32, window_height: i32, event_loop: &EventLoop) -> GlutinWindow {
        let (gl, _, window) = {
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("GG OpenGl")
                .with_inner_size(glutin::dpi::LogicalSize::new(window_width, window_height));

            unsafe {
                let window = glutin::ContextBuilder::new()
                    .with_depth_buffer(24)
                    .with_vsync(true)
                    .with_hardware_acceleration(Some(true))
                    .build_windowed(window_builder, event_loop)
                    .unwrap()
                    .make_current()
                    .unwrap();

                let gl =
                    glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
                (gl, "#version 330", window)
            }
        };


        let gl = set_gl(Arc::new(gl));

        unsafe {
            gl.viewport(0, 0, window_width, window_height);
        }

        window
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
        _object_loader: &mut ObjLoader
    ) {
        let gl = get_gl();
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::BLEND);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let default_lit = shader_loader.borrow_shader(DEFAULT_LIT_SHADER);
        light_system(ecs, default_lit, &self.lights_on);
          
        let rcs: Vec<Option<Vec<_>>> = 
            ecs.do_all(|model_handle: &Handle<Model>, transform: &Transform| {
            let model_handle = model_handle.borrow();
            if !model_handle.enabled { return None; }

            Some(
                model_handle.mesh_renderers.iter()
                .map(|mr| { RenderCommand(transform.clone(), mr.clone()) })
                .collect::<Vec<_>>()
            )
        });

        let rcs = rcs
            .iter()
            .filter_map(|v| v.as_ref())
            .flatten()
            .collect::<Vec<_>>();

        let (mut transparent, opaque): 
            (Vec<_>, Vec<_>) = rcs.into_iter().partition(|rc| rc.1.is_transparent());

        opaque.iter().for_each(|opaque_rc| {
            let RenderCommand(t, mesh) = opaque_rc;
            Self::draw_mesh(shader_loader, t, camera, mesh);
        });

        transparent.sort_by(|tr1, tr2| {
            let dist1 = (tr1.0.get_pos() - camera.get_pos()).len();
            let dist2 = (tr2.0.get_pos() - camera.get_pos()).len();

            dist1.cmp(&dist2).reverse()
        });

        transparent.iter().for_each(|tr_rc| {
            let RenderCommand(t, mesh) = tr_rc;
            Self::draw_mesh(shader_loader, t, camera, mesh);
        });
    }

    pub fn draw_mesh(shader_loader: &mut ShaderLoader, transform: &Transform, camera: &Camera, mr: &MeshRenderer) {
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

        mr.draw(shader_loader, uniforms);
    }
}

pub struct RenderCommand(Transform, MeshRenderer);


// OpenGL 
// TODO: Add outlining to objects.
// TODO: Proper transparency

// Plumbing
// TODO: Perhaps change UniformMap to an array of tuples?
// Also, add support for recursion so nested structs won't be as painful.
