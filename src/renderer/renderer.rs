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
    loaders::*,
    ecs::Ecs,
    light_system,
    transform::Transform,
    model::Model, 
    texture::Texture2D, 
    shader::{UniformMap, Uniform}, 
    map
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
        object_loader: &mut ObjLoader
    ) {
        let gl = get_gl();
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::BLEND);
            gl.clear_color(0.2, 0.2, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let default_lit = shader_loader.borrow_shader(DEFAULT_LIT_SHADER);
        light_system(ecs, default_lit, &self.lights_on);
          
        ecs.do_all::<_, _, RenderCommand>(|model_handle: &mut ModelHandle, transform: &mut Transform| {
            if !model_handle.enabled() { return None; }

            let model = object_loader.borrow(model_handle.name());
            Self::draw_model(shader_loader, transform, camera, model);

            None
        });

    }

    pub fn draw_model(shader_loader: &mut ShaderLoader, transform: &Transform, camera: &Camera, model: &mut Model) {
        let uniforms = match &model.material.material_type {
            MaterialType::Lit => map! {
                "projection" => Uniform::Mat4(camera.get_proj_matrix()),
                "view"       => Uniform::Mat4(camera.get_view_matrix()),
                "model"      => Uniform::Mat4(transform.get_model_matrix()),
                "u_view_pos" => Uniform::Vec3(camera.get_pos()),
                "u_material.shininess" => Uniform::Float(32.0)
            },
            MaterialType::Billboard | MaterialType::Unlit => map! {
                "projection" => Uniform::Mat4(camera.get_proj_matrix()),
                "view"       => Uniform::Mat4(camera.get_view_matrix()),
                "model"      => Uniform::Mat4(transform.get_model_matrix())
            },
        };

        let prefix = match &model.material.material_type {
            MaterialType::Lit  => "u_material.",
            _                          => ""
        };

        let _shader = shader_loader.borrow_shader(model.material.shader_ref()).use_program();
        model.material.upload_uniforms(shader_loader, uniforms, "");

        let shader = shader_loader.borrow_shader(model.material.shader_ref()).use_program();
        for mesh in &model.meshes {
            mesh.draw(shader, prefix);
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum MaterialType {
    Unlit,
    Lit,
    Billboard
}

#[derive(Clone, Eq)]
pub struct Material {
    shader_ref: &'static str,
    pub material_type: MaterialType
}

impl Material {
    pub fn default_billboard(texture_loader: &mut TextureLoader) -> Self {
        let directional_light = texture_loader.directional_light_texture();
        let _diffuse_texture = Texture2D::from_native_handle(
            directional_light,
            crate::texture::TextureType::Diffuse,
            1
        );

        Material {
            shader_ref   : DEFAULT_BILLBOARD_SHADER,
            material_type: MaterialType::Billboard
        }
    }

    pub fn default_unlit(_shader_loader: &mut ShaderLoader) -> Self {
        Material {
            shader_ref   : DEFAULT_UNLIT_SHADER,
            material_type: MaterialType::Unlit
        }
    }

    pub fn default_lit(_texture_loader: &mut TextureLoader) -> Self {
        Material {
            shader_ref   : DEFAULT_LIT_SHADER,
            material_type: MaterialType::Lit
        }
    }

    pub fn shader_ref(&self) -> &'static str {
        self.shader_ref
    }

    pub fn upload_uniforms(&self, shader_loader: &mut ShaderLoader, uniforms: UniformMap, prefix: &str) {
        shader_loader
            .borrow_shader(self.shader_ref())
            .upload_uniforms(uniforms, prefix);
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.shader_ref() == other.shader_ref()
    }
}

struct RenderCommand(Transform, Material, ModelHandle);


// TODO: Fix model loading (textures don't carry over when chaning materials)
// TODO: Continue moving rendering/OpenGL related code to the renderer.
// TODO: Find a way to decouple models, shaders, and textures. MeshRenderer(Model, Shader, &[Texture])? 
// TODO: Perhaps change UniformMap to an array of tuples?
// Also, add support for recursion so nested structs won't be as painful.
// TODO: Store meshes in ObjLoader
// and store only their indices for easier copying.
// TODO: Proper transparency
