extern crate egui;
extern crate egui_glow;
extern crate glow;
extern crate glutin;
extern crate image;
extern crate nalgebra_glm as glm;

use glutin::event::WindowEvent;
use light::*;
use shader::ShaderProgram;
use std::cell::RefMut;
use std::env;
use std::mem::size_of;

mod asset_loader;
mod camera;
mod ecs;
mod egui_drawable;
mod input;
mod light;
mod obj_loader;
mod scene;
mod shader;
mod texture;
mod transform;

use crate::asset_loader::TextureLoader;
use crate::obj_loader::{Model, ObjLoader, PNTVertex, PVertex, VertexAttribs};
use crate::scene::Scene;
use crate::texture::TextureType;
use ecs::ECS;
use glow::*;
use input::InputSystem;
use texture::Texture2D;
use transform::Transform;

fn light_subsystem<T: Light>(
    gl_arc: &std::rc::Rc<Context>,
    lit_shader: &mut ShaderProgram,
    transforms: &mut RefMut<Vec<Option<Transform>>>,
    spot_lights: &mut RefMut<Vec<Option<T>>>,
    u_name_light_num: &str,
    u_light_array: &str,
) {
    let enabled_count = spot_lights
        .iter()
        // Filter out None lights or disabled lights
        .filter(|l| {
            if let Some(light) = *l {
                light.is_enabled()
            } else {
                false
            }
        })
        .count() as i32;

    lit_shader.set_int(&gl_arc, u_name_light_num, enabled_count);

    let zip = spot_lights.iter_mut().zip(transforms.iter_mut());
    let mut enabled_light_index = 0;

    // Loop over all light and transform components
    // Note that some entities might have one or none. In this case light/transform
    // Will be None
    for (light, transform) in zip {
        // If an entity has both, draw egui and upload its data
        if let (Some(l), Some(t)) = (light, transform) {
            l.upload_data(
                &gl_arc,
                &t,
                &format!("{}[{}]", u_light_array, enabled_light_index),
                &lit_shader,
            );

            enabled_light_index += 1;
        }
    }
}

fn light_system(gl_rc: &std::rc::Rc<Context>, ecs: &mut ECS, lit_shader: &mut ShaderProgram) {
    if let Some(mut transforms) = ecs.borrow_comp_vec::<Transform>() {
        if let Some(mut spot_lights) = ecs.borrow_comp_vec::<SpotLight>() {
            light_subsystem::<SpotLight>(
                &gl_rc,
                lit_shader,
                &mut transforms,
                &mut spot_lights,
                "u_num_spot_lights",
                "u_spot_lights",
            );
        }

        if let Some(mut point_lights) = ecs.borrow_comp_vec::<PointLight>() {
            light_subsystem::<PointLight>(
                &gl_rc,
                lit_shader,
                &mut transforms,
                &mut point_lights,
                "u_num_point_lights",
                "u_point_lights",
            );
        }

        if let Some(mut directional_lights) = ecs.borrow_comp_vec::<DirectionalLight>() {
            let zip = directional_lights.iter_mut().zip(transforms.iter_mut());

            // Loop over all light and transform components
            // Note that some entities might have one or none. In this case light/transform
            // Will be None
            for (light, transform) in zip {
                // If an entity has both, draw egui and upload its data
                if let (Some(l), Some(t)) = (light, transform) {
                    if l.is_enabled() {
                        l.upload_data(&gl_rc, &t, "u_directional_light", &lit_shader);
                    } else {
                        DirectionalLight {
                            enabled: false,
                            colors: LightColors {
                                ambient: glm::Vec3::zeros(),
                                diffuse: glm::Vec3::zeros(),
                                specular: glm::Vec3::zeros(),
                            },
                        }.upload_data(&gl_rc, &t, "u_directional_light", &lit_shader);
                    }
                    break;
                }
            }
        }
    }
}

fn main() {
    let (window_width, window_height) = (1280, 720) as (i32, i32);

    let (gl, _, window, event_loop) = {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("GG OpenGl")
            .with_inner_size(glutin::dpi::LogicalSize::new(window_width, window_height));

        unsafe {
            let window = glutin::ContextBuilder::new()
                .with_depth_buffer(24)
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();

            let gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (gl, "#version 330", window, event_loop)
        }
    };

    let mut gl_rc = std::rc::Rc::new(gl);
    let mut egui_glow = egui_glow::EguiGlow::new(&window.window(), gl_rc.clone());

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

    let mut texture_loader = TextureLoader::new();
    let mut model = Model::load_model(&gl_rc, "assets/obj/backpack.obj", &mut texture_loader);
    model.add_texture(
        texture_loader
            .load_texture(&gl_rc, "assets/textures/grid.jpg", TextureType::Emissive)
            .1,
    );

    let mut lit_shader = shader::ShaderProgram::new(
        &gl_rc,
        "assets/shaders/textured.vert",
        "assets/shaders/lit-textured.frag",
    );

    let mut last_frame = std::time::Instant::now();
    let mut input = InputSystem::new();

    let container_pos = glm::vec3(0.0, 0.0, 0.0);
    let container_model_mat: glm::Mat4 = glm::translation(&container_pos);

    let mut scene = Scene::new(window_width, window_height);
    let mut ecs = ECS::light_test();

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

                    let view = scene.camera.get_view_matrix();

                    scene.entities_egui(&mut input, &mut egui_glow, &window, &mut ecs);

                    lit_shader.use_program(&gl_rc);
                    light_system(&mut gl_rc, &mut ecs, &mut lit_shader);
                    lit_shader.set_vec3(&gl_rc, "u_view_pos", scene.camera.get_pos());
                    lit_shader.set_float(&gl_rc, "u_material.shininess", 32.0);
                    lit_shader.set_mat4(&gl_rc, "projection", scene.get_proj_matrix());
                    lit_shader.set_mat4(&gl_rc, "view", view);
                    lit_shader.set_mat4(&gl_rc, "model", container_model_mat);
                    lit_shader.set_vec3(
                        &gl_rc,
                        "u_material.emissive_factor",
                        glm::vec3(0.1, 0.1, 0.1),
                    );
                    model.draw(&gl_rc, &lit_shader);

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
                        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
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
