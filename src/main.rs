extern crate gl;
extern crate glfw;
extern crate image;
extern crate nalgebra_glm as glm;

use glfw::Context;
use std::env;
use std::mem::size_of;

mod camera;
mod input;
mod shader;

use camera::Camera;
use input::InputSystem;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (window_width, window_height) = (1280, 720) as (i32, i32);

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::Floating(true));

    let (mut window, events) = glfw
        .create_window(
            window_width as u32,
            window_height as u32,
            "Hello, this is a window.",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_sticky_keys(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Viewport(0, 0, window_width, window_height);
        gl::ClearColor(0.2f32, 0.3f32, 0.3f32, 1.0f32);
    }

    // 3 floats for vertex position
    // 3 floats for vertex normals
    let verts = vec![
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, 0.5, -0.5, 0.0,
        0.0, -1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, -0.5,
        -0.5, 0.0, 0.0, -1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5,
        0.5, 0.5, 0.0, 0.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5,
        -0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0,
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, 0.5, -1.0,
        0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.5, 0.5, -0.5,
        1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5,
        0.5, 1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.5,
        -0.5, -0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0,
        -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, -0.5, 0.5, -0.5, 0.0,
        1.0, 0.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0,
        1.0, 0.0, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0f32,
    ];

    println!(
        "Current working directory: {}",
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    );

    let lit_shader = shader::ShaderProgram::new(
        "assets/shaders/lit-untextured.vert",
        "assets/shaders/lit-untextured.frag",
    );

    let light_shader = shader::ShaderProgram::new(
        "assets/shaders/lit-untextured.vert",
        "assets/shaders/light.frag",
    );

    let mut cube_vbo = 0u32;
    let mut lit_vao = 0u32;
    let mut light_vao = 0u32;
    unsafe {
        // Lit object setup
        lit_shader.use_program();

        gl::GenVertexArrays(1, &mut lit_vao);
        gl::BindVertexArray(lit_vao);

        gl::GenBuffers(1, &mut cube_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (size_of::<f32>() * verts.len()) as isize,
            verts.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>() as i32,
            std::ptr::null(),
        );

        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>() as i32,
            (3 * size_of::<f32>()) as *const _,
        );

        gl::EnableVertexAttribArray(1);

        light_shader.use_program();

        // Light setup
        gl::GenVertexArrays(1, &mut light_vao);
        gl::BindVertexArray(light_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo); // Reuse the vbo from the cube

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>() as i32, // Skip texture coordinates and normals
            std::ptr::null(),
        );

        gl::EnableVertexAttribArray(0);

        gl::Enable(gl::DEPTH_TEST);
    }

    let mut last_frame = 0.0f32;

    let mut input = InputSystem::new();
    let mut camera = Camera::new(
        &glm::vec3(0.0, 0.0, 2.0f32),
        &glm::vec3(0.0, 1.0, 0.0f32),
        &glm::vec2(0.0, 0.0),
    );

    let container_pos = glm::vec3(0.0, 0.0, 0.0);
    let container_model_mat = glm::translation(&container_pos);

    let light_pos = glm::vec3(0.6, 0.9, 0.7);
    let mut light_model_mat = glm::translation(&light_pos);
    light_model_mat = glm::scale(&light_model_mat, &glm::vec3(0.05, 0.05, 0.05));

    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;

        glfw.poll_events();
        let mut glfw_events = std::vec::Vec::<glfw::WindowEvent>::new();
        for (_, event) in glfw::flush_messages(&events) {
            glfw_events.push(event);
        }

        input.handle_glfw(glfw_events, &(current_frame - last_frame));
        camera.update(&input);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let view = camera.get_view_matrix();

            let projection = glm::perspective_fov(
                camera.get_fov().to_radians(),
                window_width as f32,
                window_height as f32,
                0.1f32,
                100.0f32,
            );

            lit_shader.use_program();
            lit_shader.set_vec3("u_view_pos", camera.get_pos());

            lit_shader.set_vec3("u_material.ambient", glm::vec3(1.0, 0.5, 0.31));
            lit_shader.set_vec3("u_material.diffuse", glm::vec3(1.0, 0.5, 0.31));
            lit_shader.set_vec3("u_material.specular", glm::vec3(0.5, 0.5, 0.5));
            lit_shader.set_float("u_material.shininess", 32.0);

            lit_shader.set_vec3("u_light.ambient", glm::vec3(0.2, 0.2, 0.2));
            lit_shader.set_vec3("u_light.diffuse", glm::vec3(0.5, 0.5, 0.5)); // darken diffuse light a bit
            lit_shader.set_vec3("u_light.specular", glm::vec3(1.0, 1.0, 1.0));
            lit_shader.set_vec3("u_light.position", light_pos);

            lit_shader.set_mat4("projection", projection);
            lit_shader.set_mat4("view", view);
            lit_shader.set_mat4("model", container_model_mat);

            gl::BindVertexArray(lit_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            light_shader.use_program();
            light_shader.set_mat4("projection", projection);
            light_shader.set_mat4("view", view);
            light_shader.set_mat4("model", light_model_mat);
            gl::BindVertexArray(light_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        last_frame = current_frame;
        window.swap_buffers();
    }
}
