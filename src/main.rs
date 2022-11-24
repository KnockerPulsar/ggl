use egui_glow::EguiGlow;
use glutin::{event::{WindowEvent, VirtualKeyCode}, ContextWrapper};
use std::env;

mod asset_loader;
mod camera;
mod ecs;
mod egui_drawable;
mod input;
mod light;
mod light_system;
mod obj_loader;
mod scene;
mod shader;
mod shader_loader;
mod texture;
mod transform;
mod gl;

use crate::asset_loader::TextureLoader;
use glow::HasContext;
use obj_loader::Model;
use scene::Scene;
use ecs::Ecs;
use input::InputSystem;
use light_system::*;
use shader_loader::ShaderLoader;
use transform::Transform;
use gl::{ set_gl, get_gl };

use nalgebra_glm::*;


fn render_system(
    scene: &mut Scene,
    mut shader_loader: &mut ShaderLoader,
    mut ecs: &mut Ecs,
    lights_on: &mut bool,
) {
    let view = scene.camera.get_view_matrix();

    let lit_shader = shader_loader.borrow_shader("default").unwrap();

    lit_shader.use_program();

    light_system(&mut ecs, lit_shader, &lights_on);

    lit_shader.set_vec3("u_view_pos", scene.camera.get_pos());
    lit_shader.set_float("u_material.shininess", 32.0);
    lit_shader.set_mat4("projection", scene.get_proj_matrix());
    lit_shader.set_mat4("view", view);
    lit_shader.set_vec3(
        "u_material.emissive_factor",
        vec3(0.1, 0.1, 0.1),
    );

    ecs.do_all::<Model, Transform>(|model, transform| {
        model.draw(&mut shader_loader, &transform);
    });


}

fn main() {
    let (window_width, window_height) = (1280, 720) as (i32, i32);

    let (gl, _, window, event_loop) = {
        let event_loop: glutin::event_loop::EventLoop<()> = glutin::event_loop::EventLoopBuilder::with_user_event().build();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("GG OpenGl")
            .with_inner_size(glutin::dpi::LogicalSize::new(window_width, window_height));

        unsafe {
            let window = glutin::ContextBuilder::new()
                .with_depth_buffer(24)
                .with_vsync(true)
                .with_hardware_acceleration(Some(true))
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();

            let gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (gl, "#version 330", window, event_loop)
        }
    };

    
    let gl_rc = set_gl(std::sync::Arc::new(gl));

    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, get_gl().clone());

    unsafe {
        gl_rc.viewport(0, 0, window_width, window_height);
    }

    println!(
        "Current working directory: {}",
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    );

    println!("Loading, please wait...");

    let shaders = [
        ("lit-textured",
         "assets/shaders/textured.vert",
         "assets/shaders/lit-textured.frag"),
    ];

    let default_textures = [
        "assets/textures/white.jpeg",
        "assets/textures/black.jpg",
        "assets/textures/grid.jpg"
    ];

    let mut shader_loader = ShaderLoader::new(&shaders);
    let mut texture_loader = TextureLoader::new(&default_textures);

    let mut last_frame = std::time::Instant::now();
    let mut input = InputSystem::new();

    let mut scene = Scene::new(window_width, window_height);
    let mut ecs = Ecs::light_test();

    let _model = ecs
        .add_entity()
        .with(Transform::new(
                vec3(0.0, 0.0, -2.0),
                Vec3::zeros(),
                "model",
        ))
        .with(Model::load("assets/obj/cube.obj", &mut texture_loader));

    let mut lights_on = true;
    unsafe {
        event_loop.run(
            move |event, _, control_flow: &mut glutin::event_loop::ControlFlow| {
                let mut redraw = || {
                    gl_rc.enable(glow::DEPTH_TEST);
                    gl_rc.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);


                    // draw things behind egui here
                    let current_frame = std::time::Instant::now();
                    input.update((current_frame - last_frame).as_secs_f32());
                    scene.camera.update(&mut input);

                    render_system(&mut scene, &mut shader_loader, &mut ecs, &mut lights_on);


                    egui_glow.run(window.window(), |egui_ctx| {
                        scene.entities_egui(&mut input, egui_ctx, &mut ecs);

                        egui::Window::new("Global Light Toggle").show(egui_ctx, |ui| {
                            ui.checkbox(&mut lights_on, "Lights on?");
                        });

                        
                        egui::Window::new("Load Model").show(egui_ctx, |ui| {
                            if ui.button("Load").clicked() {
                                let path = rfd::FileDialog::new().add_filter("Object model", &["obj"]).pick_file(); 
                                if let Some(path) = path {
                                    let str_path = path.to_str().unwrap();
                                    let mut transform = Transform::zeros();
                                    transform.set_name(str_path);

                                    ecs
                                        .add_entity()
                                        .with(transform)
                                        .with(Model::load(str_path, &mut texture_loader));
                                }
                            }
                        });
                    });


                    egui_glow.paint(window.window());

                    // draw things on top of egui here
                    last_frame = current_frame;
                    input.frame_end();

                    window.swap_buffers().unwrap();
                };

                match event {
                    // Platform-dependent event handlers to workaround a winit bug
                    // See: https://github.com/rust-windowing/winit/issues/987
                    // See: https://github.com/rust-windowing/winit/issues/1619
                    glutin::event::Event::MainEventsCleared if !cfg!(windows) => {
                        redraw();
                    }

                    glutin::event::Event::WindowEvent { event, .. } => {
                        if 
                            matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) 
                                || input.just_pressed(VirtualKeyCode::Escape) {
                            *control_flow = glutin::event_loop::ControlFlow::Exit;
                        }

                        if let glutin::event::WindowEvent::Resized(physical_size) = &event {
                            window.resize(*physical_size);
                            scene.window_size_changed(physical_size);
                            gl_rc.viewport(0, 0, window_width, window_height);
                        } else if let glutin::event::WindowEvent::ScaleFactorChanged {
                            new_inner_size,
                            ..
                        } = &event
                        {
                            window.resize(**new_inner_size);
                        }

                        egui_glow.on_event(&event);

                        match event {
                            WindowEvent::KeyboardInput { .. }
                            | WindowEvent::CursorMoved { .. }
                            | WindowEvent::MouseInput { .. } => {
                                input.handle_events(&event);
                            }
                            _ => {}
                        }

                        window.window().request_redraw(); // TODO(emilk): ask egui if the events warrants a repaint instead
                    }
                    glutin::event::Event::LoopDestroyed => {
                        egui_glow.destroy();
                    }

                    _ => (),
                }

            },
        );
    }
}
