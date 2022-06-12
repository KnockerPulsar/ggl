extern crate egui;
extern crate egui_glow;
extern crate glow;
extern crate glutin;
extern crate image;
extern crate nalgebra_glm as glm;

use glutin::event::WindowEvent;
use light::*;
use std::env;
use std::mem::size_of;

mod camera;
mod egui_drawable;
mod input;
mod light;
mod scene;
mod shader;
mod texture;
mod transform;

use camera::Camera;
use egui_drawable::EguiDrawable;
use glow::*;
use input::InputSystem;
use scene::Scene;
use texture::Texture2D;
use transform::Transform;

fn ecs_test_scene() -> Scene {
    let mut s = Scene::new();

    let e1 = s.add_entity();
    s.add_comp_to_entity(
        e1,
        Transform::new(
            glm::vec3(3.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, -90.0f32.to_radians()),
            "Light 1",
        ),
    )
    .add_comp_to_entity(
        e1,
        SpotLight {
            enabled: true,
            colors: LightColors {
                ambient: glm::vec3(0.1f32, 0.0, 0.0),
                diffuse: glm::vec3(10.0, 0.0, 0.0),
                specular: glm::vec3(0.0, 10.0, 10.0),
            },
            attenuation_constants: glm::vec3(1.0, 0.0, 1.0),
            cutoff_cosines: glm::vec2(2.5f32.to_radians().cos(), 5f32.to_radians().cos()),
        },
    );

    s
}

fn main() {
    let (window_width, window_height) = (1280, 720) as (i32, i32);

    let (gl, shader_version, window, event_loop) = {
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

    let gl_arc = std::sync::Arc::new(gl);
    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl_arc.clone());

    unsafe {
        gl_arc.viewport(0, 0, window_width, window_height);
    }

    // 3 floats for vertex position
    // 3 floats for vertex normals
    // 2 floats for texture coordinates
    let verts = vec![
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 0.0, 0.5,
        0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, -0.5, 0.5,
        -0.5, 0.0, 0.0, -1.0, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, -0.5, -0.5,
        0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0,
        0.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0,
        0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0,
        0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, 1.0, 1.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0,
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0, -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, 0.0, 0.0,
        -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5,
        -0.5, 1.0, 0.0, 0.0, 1.0, 1.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, -0.5,
        1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
        0.0, 1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, -1.0, 0.0,
        1.0, 1.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0,
        0.0, -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0,
        -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, 0.5,
        0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, -0.5, 0.5, 0.5,
        0.0, 1.0, 0.0, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0f32,
    ];

    println!(
        "Current working directory: {}",
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    );

    let container_diff = Texture2D::load(&gl_arc, "assets/textures/container2.png");
    let container_spec = Texture2D::load(&gl_arc, "assets/textures/container2_specular.png");
    let container_emissive = Texture2D::load(&gl_arc, "assets/textures/container2_emissive.png");

    let lit_shader = shader::ShaderProgram::new(
        &gl_arc,
        "assets/shaders/lit-untextured.vert",
        "assets/shaders/lit-untextured.frag",
    );

    let light_shader = shader::ShaderProgram::new(
        &gl_arc,
        "assets/shaders/lit-untextured.vert",
        "assets/shaders/light.frag",
    );

    let cube_vbo: glow::Buffer;
    let lit_vao: glow::VertexArray;
    let light_vao: glow::VertexArray;
    unsafe {
        // Lit object setup
        lit_shader.use_program(&gl_arc);

        lit_vao = gl_arc.create_vertex_array().unwrap();
        gl_arc.bind_vertex_array(Some(lit_vao));

        cube_vbo = gl_arc.create_buffer().unwrap();
        gl_arc.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));
        gl_arc.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            std::slice::from_raw_parts(verts.as_ptr() as *const u8, size_of::<f32>() * verts.len()),
            glow::STATIC_DRAW,
        );

        gl_arc.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 8 * size_of::<f32>() as i32, 0);
        gl_arc.enable_vertex_attrib_array(0);

        gl_arc.vertex_attrib_pointer_f32(
            1,
            3,
            glow::FLOAT,
            false,
            8 * size_of::<f32>() as i32,
            3 * size_of::<f32>() as i32,
        );
        gl_arc.enable_vertex_attrib_array(1);

        gl_arc.vertex_attrib_pointer_f32(
            2,
            2,
            glow::FLOAT,
            false,
            8 * size_of::<f32>() as i32,
            6 * size_of::<f32>() as i32,
        );
        gl_arc.enable_vertex_attrib_array(2);

        light_shader.use_program(&gl_arc);

        // Light setup

        light_vao = gl_arc.create_vertex_array().unwrap();
        gl_arc.bind_vertex_array(Some(light_vao));
        gl_arc.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));

        gl_arc.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 8 * size_of::<f32>() as i32, 0);
        gl_arc.enable_vertex_attrib_array(0);

        gl_arc.enable(glow::TEXTURE_2D);
    }

    let mut last_frame = std::time::Instant::now();

    let mut input = InputSystem::new();
    let mut camera = Camera::new(
        &glm::vec3(0.0, 0.0, 2.0f32),
        &glm::vec3(0.0, 1.0, 0.0f32),
        &glm::vec2(0.0, 0.0),
    );

    let container_pos = glm::vec3(0.0, 0.0, 0.0);
    let container_model_mat = glm::translation(&container_pos);

    let light_pos = glm::vec3(3.0, 0.0, 0.0);
    let mut light_model_mat = glm::translation(&light_pos);
    light_model_mat = glm::scale(&light_model_mat, &glm::vec3(0.05, 0.05, 0.05));
    let mut scene = ecs_test_scene();

    unsafe {
        event_loop.run(
            move |event, _, control_flow: &mut glutin::event_loop::ControlFlow| {
                let mut redraw = || {
                    gl_arc.enable(glow::DEPTH_TEST);

                    gl_arc.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                    // draw things behind egui here
                    let current_frame = std::time::Instant::now();

                    input.update((current_frame - last_frame).as_secs_f32());
                    camera.update(&mut input);

                    {
                        let view = camera.get_view_matrix();
                        let projection = glm::perspective_fov(
                            camera.get_fov().to_radians(),
                            window_width as f32,
                            window_height as f32,
                            0.1f32,
                            100.0f32,
                        );

                        lit_shader.use_program(&gl_arc);

                        lit_shader.set_vec3(&gl_arc, "u_view_pos", camera.get_pos());

                        container_diff.use_texture(&gl_arc, 0, "u_material.diffuse", &lit_shader);
                        container_spec.use_texture(&gl_arc, 1, "u_material.specular", &lit_shader);
                        container_emissive.use_texture(
                            &gl_arc,
                            2,
                            "u_material.emissive",
                            &lit_shader,
                        );

                        lit_shader.set_float(&gl_arc, "u_material.shininess", 32.0);

                        if let (Some(mut spot_lights), Some(mut transforms)) = (
                            scene.borrow_comp_vec::<SpotLight>(),
                            scene.borrow_comp_vec::<Transform>(),
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

                            lit_shader.set_int(&gl_arc, "u_num_spot_lights", enabled_count);

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
                                        &format!("u_spot_lights[{}]", enabled_light_index),
                                        &lit_shader,
                                    );

                                    enabled_light_index += 1;
                                }
                            }
                        }

                        egui_glow.run(window.window(), |egui_ctx| {
                            egui::Window::new("spot_lights").show(egui_ctx, |ui| {
                                scene.on_egui(ui);
                            });
                        });

                        lit_shader.set_mat4(&gl_arc, "projection", projection);
                        lit_shader.set_mat4(&gl_arc, "view", view);
                        lit_shader.set_mat4(&gl_arc, "model", container_model_mat);

                        gl_arc.bind_vertex_array(Some(lit_vao));
                        gl_arc.draw_arrays(glow::TRIANGLES, 0, 36);

                        light_shader.use_program(&gl_arc);
                        light_shader.set_mat4(&gl_arc, "projection", projection);
                        light_shader.set_mat4(&gl_arc, "view", view);
                        light_shader.set_mat4(&gl_arc, "model", light_model_mat);

                        gl_arc.bind_vertex_array(Some(light_vao));
                        gl_arc.draw_arrays(glow::TRIANGLES, 0, 36);
                    };

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
