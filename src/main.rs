extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use std::mem::size_of;

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

    let verts = vec![-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0f32];
    let vert_shader_src = r#"
        #version 330 core
        layout (location = 0) in vec3 aPos;
    
        void main() {
            gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
        }
    "#;
    let mut vert_shader = create_shader(vert_shader_src, gl::VERTEX_SHADER);

    let frag_shader_src = r#"
        #version 330 core
    
        out vec4 FragColor;
    
        void main(){
            FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
        }
    "#;

    let mut frag_shader = create_shader(frag_shader_src, gl::FRAGMENT_SHADER);

    let mut shader_program = create_program(vert_shader, frag_shader);

    let mut vao = 0u32;
    let mut vbo = 0u32;
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

        gl::UseProgram(shader_program);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * size_of::<f32>() as i32,
            std::ptr::null(),
        );

        gl::EnableVertexAttribArray(0);
    }

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        _ => {}
    }
}

fn create_shader(shader_src: &str, shader_type: GLenum) -> u32 {
    unsafe {
        let shader_id = gl::CreateShader(shader_type);
        gl::ShaderSource(
            shader_id,
            1,
            &(shader_src.as_bytes().as_ptr().cast()),
            &(shader_src.len().try_into().unwrap()),
        );
        gl::CompileShader(shader_id);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

        if success != (gl::TRUE as GLint) {
            let mut len = 0;

            gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize - 1);

            gl::GetShaderInfoLog(
                shader_id,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{}",
                std::str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInforLog not vlid utf8")
            )
        } else {
            shader_id
        }
    }
}

fn create_program(vert_shader_id: u32, frag_shader_id: u32) -> u32 {
    unsafe {
        let mut shader_program = 0u32;
        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vert_shader_id);
        gl::AttachShader(shader_program, frag_shader_id);
        gl::LinkProgram(shader_program);

        let mut success = gl::FALSE as GLint;

        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);

        if success != (gl::TRUE as GLint) {
            let mut len = 0;

            gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize - 1);

            gl::GetProgramInfoLog(
                shader_program,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{}",
                std::str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInforLog not vlid utf8")
            )
        } else {
            gl::DeleteShader(vert_shader_id);
            gl::DeleteShader(frag_shader_id);
            shader_program
        }
    }
}
