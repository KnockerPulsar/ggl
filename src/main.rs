extern crate gl;
extern crate glfw;

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
        // top right
        0.5, 0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, // top left
        -0.5, 0.5, 0.0, 0.5, 0.5, 0.5f32,
    ];

    let inds = vec![0, 1, 3, 1, 2, 3];

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

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (size_of::<f32>() * inds.len()) as isize,
            inds.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    }
    let mut polygon_mode = false;
    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut polygon_mode);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            shader_program.use_program();
            gl::BindVertexArray(vao);

            let time = glfw.get_time();
            let green = ((time.sin() / 2.0) + 0.5) as f32;
            gl::Uniform4f(
                gl::GetUniformLocation(shader_program.id, "u_color".as_bytes().as_ptr().cast()),
                0.0f32,
                green,
                0.0f32,
                1.0f32,
            );

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
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
