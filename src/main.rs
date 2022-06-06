extern crate gl;
extern crate glfw;
extern crate image;
extern crate nalgebra_glm as glm;

use glfw::{Action, Context, Key};
use std::env;
use std::mem::size_of;

mod shader;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(
            300,
            300,
            "Hello, this is a window.",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Viewport(0, 0, 300, 300);
        gl::ClearColor(0.2f32, 0.3f32, 0.3f32, 1.0f32);
    }

    let verts = vec![
        -0.5, -0.5, -0.5, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5,
        -0.5, 1.0, 1.0, -0.5, 0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 0.0, -0.5, -0.5, 0.5,
        0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0, -0.5,
        0.5, 0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5, -0.5,
        1.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0,
        0.0, -0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5,
        -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 0.5, 0.5, 0.5,
        1.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0,
        0.5, -0.5, 0.5, 1.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, 0.5,
        -0.5, 0.0, 1.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0f32,
    ];

    println!(
        "Current working directory: {}",
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    );

    let shader_program =
        shader::ShaderProgram::new("assets/shaders/simple.vert", "assets/shaders/simple.frag");

    let mut vao = 0u32;
    let mut vbo = 0u32;
    let mut ebo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (size_of::<f32>() * verts.len()) as isize,
            verts.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        shader_program.use_program();

        gl::VertexAttribPointer(
            0,                           // Atribute location
            3,                           // Number of elements to send
            gl::FLOAT,                   // Element type
            gl::FALSE,                   // Normalized? (for converting ints to floats)
            5 * size_of::<f32>() as i32, // Stride between each attribute group
            std::ptr::null(),            // Offset to read the first group from
        );

        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            5 * size_of::<f32>() as i32,
            (3 * size_of::<f32>()) as *const _,
        );

        gl::EnableVertexAttribArray(1);

        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::Enable(gl::DEPTH_TEST);
    }

    let container = image::io::Reader::open("assets/textures/container.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let awesomeface = image::io::Reader::open("assets/textures/awesomeface.png")
        .unwrap()
        .decode()
        .unwrap()
        .flipv();

    let container_w = container.width() as i32;
    let container_h = container.height() as i32;

    let awesome_w = awesomeface.width() as i32;
    let awesome_h = awesomeface.height() as i32;

    let mut container_id = 0;
    let mut awesome_id = 0;
    unsafe {
        gl::GenTextures(1, &mut container_id);
        gl::BindTexture(gl::TEXTURE_2D, container_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            container_w,
            container_h,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            container.as_bytes().as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::GenTextures(1, &mut awesome_id);
        gl::BindTexture(gl::TEXTURE_2D, awesome_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            awesome_w,
            awesome_h,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            awesomeface.as_bytes().as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    let cube_positions = vec![
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(2.0, 5.0, -15.0),
        glm::vec3(-1.5, -2.2, -2.5),
        glm::vec3(-3.8, -2.0, -12.3),
        glm::vec3(2.4, -0.4, -3.5),
        glm::vec3(-1.7, 3.0, -7.5),
        glm::vec3(1.3, -2.0, -2.5),
        glm::vec3(1.5, 2.0, -2.5),
        glm::vec3(1.5, 0.2, -1.5),
        glm::vec3(-1.3, 1.0, -1.5),
    ];

    let mut polygon_mode = false;
    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut polygon_mode);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, container_id);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, awesome_id);

            shader_program.use_program();
            gl::BindVertexArray(vao);

            // Set sampler `tex1` to read from texture unit 0
            shader_program.set_int("texture1", 0);
            shader_program.set_int("texture2", 1);

            let radius = 10.0f64;
            let cam_x = (glfw.get_time().sin() * radius) as f32;
            let cam_z = (glfw.get_time().cos() * radius) as f32;

            let camera_pos = glm::vec3(cam_x, 0.0f32, cam_z);
            let camera_target = glm::vec3(0.0, 0.0, 0.0f32);
            let up = glm::vec3(0.0, 1.0, 0.0f32);
            let view = glm::look_at(&camera_pos, &camera_target, &up);

            let projection =
                glm::perspective_fov(45.0f32.to_radians(), 300f32, 300f32, 0.1f32, 100.0f32);

            shader_program.set_mat4("view", view);
            shader_program.set_mat4("projection", projection);

            for i in 0..10 {
                let mut model = glm::translation(&cube_positions[i]);

                let angle = 20.0f32 * i as f32;

                model = glm::rotate(
                    &model,
                    angle.to_radians(),
                    &glm::make_vec3::<f32>(&[1.0, 0.3, 0.5f32]),
                );

                shader_program.set_mat4("model", model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
            // gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.swap_buffers();
    }
}

fn handle_window_event(
    window: &mut glfw::Window,
    event: glfw::WindowEvent,
    polygon_mode: &mut bool,
) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        glfw::WindowEvent::Key(Key::F, _, Action::Press, _) => {
            *polygon_mode = !(*polygon_mode);

            unsafe {
                if *polygon_mode {
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                } else {
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                }
            }
        }
        _ => {}
    }
}
