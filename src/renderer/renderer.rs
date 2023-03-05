use std::sync::Arc;

use glow::HasContext;
use glutin::dpi::PhysicalSize;

use crate::{
    gl::{set_gl, get_gl},
    app::EventLoop,
    camera::Camera,
    loaders::*,
    ecs::Ecs,
    light_system,
    transform::Transform,
    model::ModelType, texture::Texture2D, shader::ShaderProgram
};

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
        object_loader: &ObjLoader
    ) {
        let gl = get_gl();
        unsafe{
            gl.enable(glow::DEPTH_TEST);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::BLEND);
            gl.clear_color(0.9, 0.9, 0.9, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let view = camera.get_view_matrix();
        let lit_shader = shader_loader.borrow_shader("default").unwrap();
        lit_shader.use_program();

        light_system(ecs, lit_shader, &mut self.lights_on);

        lit_shader
            .set_vec3("u_view_pos", camera.get_pos())
            .set_float("u_material.shininess", 32.0)
            .set_mat4("projection", camera.get_proj_matrix())
            .set_mat4("view", view);

        // lit_shader.set_float("u_material.emissive_factor", 1.0);

        ecs.do_all::<ModelHandle, Transform>(|model_handle, transform| {
            if !model_handle.enabled() { return; }

            let model = object_loader.borrow(model_handle.name());
            match model.model_type {
                ModelType::Normal => {
                    model.draw_normal(shader_loader, transform)
                },
                ModelType::Billboard => model.draw_billboard(shader_loader, transform, camera),
            };

        });

    }
}


// #[derive(Debug)]
// struct MeshRenderer {
//     model: ModelHandle,
//     shader: ShaderProgram,
//     textures: Vec<Texture2D>,
// }
//
//
// enum RenderCommand {
//     RenderMesh(MeshRenderer),
//     RenderBillboard(BillboardRenderer)
// }

// TODO: Continue moving rendering/OpenGL related code to the renderer.
// TODO: Find a way to decouple models, shaders, and textures. MeshRenderer(Model, Shader, &[Texture])? 
