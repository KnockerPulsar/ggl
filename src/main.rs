extern crate gl;
extern crate glfw;
extern crate image;

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
        0.5, 0.5, 0.0, // top right
        1.0, 0.0, 0.0, // Vertex colors
        1.0, 1.0, // Texture coords
        0.5, -0.5, 0.0, // bottom right
        0.0, 1.0, 0.0, // Vertex colors
        1.0, 0.0, // Texture coords
        -0.5, -0.5, 0.0, // bottom left
        0.0, 0.0, 1.0, // Vertex colors
        0.0, 0.0, // Texture coords
        -0.5, 0.5, 0.0, // top left
        0.5, 0.5, 0.5, // Vertex Colors
        0.0, 1.0f32, // Texture coords
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
            0,                           // Atribute location
            3,                           // Number of elements to send
            gl::FLOAT,                   // Element type
            gl::FALSE,                   // Normalized? (for converting ints to floats)
            8 * size_of::<f32>() as i32, // Stride between each attribute group
            std::ptr::null(),            // Offset to read the first group from
        );

        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * size_of::<f32>() as i32,
            (3 * size_of::<f32>()) as *const _,
        );

        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            8 * size_of::<f32>() as i32,
            (6 * size_of::<f32>()) as *const _,
        );

        gl::EnableVertexAttribArray(2);

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

    let mut polygon_mode = false;
    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut polygon_mode);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, container_id);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, awesome_id);

            shader_program.use_program();
            gl::BindVertexArray(vao);

            // Set sampler `tex1` to read from texture unit 0
            shader_program.set_int("texture1", 0);
            shader_program.set_int("texture2", 1);

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
