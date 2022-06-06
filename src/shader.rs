extern crate gl;

use gl::types::*;
use nalgebra_glm as glm;
use std::fs;

pub struct ShaderProgram {
    pub id: u32,
}

impl ShaderProgram {
    pub fn new(vert_shader_path: &str, frag_shader_path: &str) -> ShaderProgram {
        let vert_shader_src = fs::read_to_string(vert_shader_path).expect(&format!(
            "Failed to read vertex shader at {}",
            vert_shader_path
        ));
        let frag_shader_src = fs::read_to_string(frag_shader_path).expect(&format!(
            "Failed to read fragment shader at {}",
            vert_shader_path
        ));

        let vert_shader_id = create_shader(&vert_shader_src, gl::VERTEX_SHADER);
        let frag_shader_id = create_shader(&frag_shader_src, gl::FRAGMENT_SHADER);

        let shader_program_id = create_program(vert_shader_id, frag_shader_id);

        ShaderProgram {
            id: shader_program_id,
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    fn get_uniform_location(&self, name: &str) -> i32 {
        unsafe {
            let name_cstr = std::ffi::CString::new(name).expect("CString::new failed");
            let location = gl::GetUniformLocation(self.id, name_cstr.as_ptr().cast());

            if location == -1 {
                panic!(
                    "The requested uniform {} is not in the shader with id {}",
                    name, self.id
                );
            } else {
                location
            }
        }
    }

    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.get_uniform_location(name), value as i32);
        }
    }

    // GLSL doesn't have bools?
    pub fn set_bool(&self, name: &str, value: bool) {
        self.set_int(name, value as i32);
    }

    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(self.get_uniform_location(name), value);
        }
    }

    pub fn set_mat4(&self, name: &str, value: glm::Mat4) {
        unsafe {
            gl::UniformMatrix4fv(
                self.get_uniform_location(name),
                1,
                gl::FALSE,
                glm::value_ptr(&value).as_ptr().cast(),
            );
        }
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
