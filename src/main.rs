

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
use obj_loader::{Model, ObjLoader, ModelHandle};
use scene::Scene;
use ecs::Ecs;
use input::InputSystem;
use light_system::*;
use shader_loader::ShaderLoader;
use transform::Transform;
use gl::{ set_gl, get_gl };

use egui_gizmo::GizmoMode;
use glutin::event::{WindowEvent, VirtualKeyCode};

use std::env;
use std::sync::Arc;

fn render_system(
    scene: &mut Scene,
    shader_loader: &mut ShaderLoader,
    object_loader: &mut ObjLoader,
    ecs: &mut Ecs,
    lights_on: &mut bool,
) {
    let view = scene.camera.get_view_matrix();

    let lit_shader = shader_loader.borrow_shader("default").unwrap();

    lit_shader.use_program();

    light_system(ecs, lit_shader, lights_on);

    lit_shader.set_vec3("u_view_pos", scene.camera.get_pos());
    lit_shader.set_float("u_material.shininess", 32.0);
    lit_shader.set_mat4("projection", scene.get_proj_matrix());
    lit_shader.set_mat4("view", view);
    // lit_shader.set_float("u_material.emissive_factor", 1.0);
    

    ecs.do_all::<ModelHandle, Transform>(|model_handle, transform| {
        let model = object_loader.borrow(model_handle).unwrap();
        model.draw(shader_loader, transform);
    });
}

type GlutinWindow = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

fn init(window_width: i32, window_height: i32) -> (
    Arc<glow::Context>, 
    GlutinWindow, 
    glutin::event_loop::EventLoop<()>,
    egui_glow::EguiGlow
) {

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

    
    let gl = set_gl(Arc::new(gl));
    let egui_glow = egui_glow::EguiGlow::new(&event_loop, Arc::clone(get_gl()));

    unsafe {
        gl.viewport(0, 0, window_width, window_height);
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

    (Arc::clone(gl), window, event_loop, egui_glow)
}

fn handle_events(
    event: glutin::event::Event<()>, 
    control_flow: &mut glutin::event_loop::ControlFlow,
    input: &mut InputSystem,
    window: &GlutinWindow,
    window_width: i32,
    window_height: i32,
    scene: &mut Scene,
    egui_glow: &mut egui_glow::EguiGlow
) {
    let gl = get_gl();

    match event {
        glutin::event::Event::WindowEvent { event, .. } => {

            // Close window
            if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) 
                || input.just_pressed(VirtualKeyCode::Escape) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
            }

            // Resize window
            if let glutin::event::WindowEvent::Resized(physical_size) = &event {
                window.resize(*physical_size);
                scene.window_size_changed(physical_size);

                unsafe {
                    gl.viewport(0, 0, window_width, window_height);
                }
            } else if let glutin::event::WindowEvent::ScaleFactorChanged {
                new_inner_size,
                ..
            } = &event
            {
                window.resize(**new_inner_size);
            }

            egui_glow.on_event(&event);

            // Input event
            if let 
                WindowEvent::KeyboardInput { .. } 
                | WindowEvent::CursorMoved { .. } 
                | WindowEvent::MouseInput { .. } 
                = event {
                input.handle_events(&event);
            }


            window.window().request_redraw(); // TODO(emilk): ask egui if the events warrants a repaint instead
        }

        glutin::event::Event::LoopDestroyed => {
            egui_glow.destroy();
        }

        _ => (),
    }
}

fn egui_ui(
    egui_glow: &mut egui_glow::EguiGlow,
    window: &GlutinWindow,
    mut ecs: &mut Ecs,
    scene: &mut Scene,
    input: &mut InputSystem,
    mut lights_on: &mut bool,
    mut texture_loader: &mut TextureLoader,
    object_loader: &mut ObjLoader
) {
    egui_glow.run(window.window(), |egui_ctx| {
        egui::Window::new("ggl").show(egui_ctx, |ui| {


            ui.spacing();
            ui.heading("Entities");
            ui.group(|ui| {
                scene.entities_egui(ui, &mut ecs);
            });

            scene.selected_entity_gizmo(egui_ctx, &mut ecs);

            if input.is_down(glutin::event::VirtualKeyCode::T) {
                scene.gizmo_mode = GizmoMode::Translate;
            }

            if input.is_down(glutin::event::VirtualKeyCode::R) {
                scene.gizmo_mode = GizmoMode::Rotate;
            }

            if input.is_down(glutin::event::VirtualKeyCode::Y) {
                scene.gizmo_mode = GizmoMode::Scale;
            }

            ui.spacing();
            ui.heading("Global light toggle");

            ui.group(|ui| {
                ui.checkbox(&mut lights_on, "Lights on?");
            });

            ui.spacing();
            ui.heading("Load models");

            ui.group(|ui| {
                if ui.button("Load").clicked() {
                    let path = rfd::FileDialog::new().add_filter("Object model", &["obj"]).pick_file(); 
                    if let Some(path) = path {
                        let str_path = path.to_str().unwrap();
                        let mut transform = Transform::zeros();
                        transform.set_name(str_path);

                        ecs
                            .add_entity()
                            .with(transform)
                            .with(object_loader.load(str_path, &mut texture_loader).unwrap());
                    }
                }
            });
        });
    });


    egui_glow.paint(window.window());
}

fn main() {
    let (window_width, window_height) = (1280, 720) as (i32, i32);
    let (gl, window, event_loop, mut egui_glow) = init(window_width, window_height);

    let custom_shaders = [
        ("lit-textured",
         "assets/shaders/textured.vert",
         "assets/shaders/lit-textured.frag"),
    ];

    let mut shader_loader = ShaderLoader::new(&custom_shaders);
    let mut texture_loader = TextureLoader::new();
    let mut object_loader = ObjLoader::new();

    let mut last_frame = std::time::Instant::now();
    let mut input = InputSystem::new();

    let mut scene = Scene::new(window_width, window_height);
    let mut ecs = Ecs::light_test(&mut texture_loader, &mut object_loader);

    let mut lights_on = true;

    unsafe {
        event_loop.run( move |event, _, control_flow| {

            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            if let glutin::event::Event::MainEventsCleared = event {
                if !cfg!(windows) {
                    gl.enable(glow::DEPTH_TEST);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);


                    // draw things behind egui here
                    let current_frame = std::time::Instant::now();
                    input.update((current_frame - last_frame).as_secs_f32());
                    scene.camera.update(&mut input);

                    render_system(
                        &mut scene,
                        &mut shader_loader,
                        &mut object_loader,
                        &mut ecs,
                        &mut lights_on
                    );

                    egui_ui(
                        &mut egui_glow,
                        &window,
                        &mut ecs,
                        &mut scene,
                        &mut input,
                        &mut lights_on,
                        &mut texture_loader,
                        &mut object_loader
                    );

                    // draw things on top of egui here
                    last_frame = current_frame;
                    input.frame_end();

                    window.swap_buffers().unwrap();

                }
            }

            handle_events(
                event,
                control_flow,
                &mut input,
                &window,
                window_width,
                window_height,
                &mut scene,
                &mut egui_glow,
            );

        }
        );
    }
}
