use glow::*;
use nalgebra_glm as glm;
use std::fs;
use std::rc::Rc;

pub struct ShaderProgram {
    pub handle: glow::Program,
}

impl ShaderProgram {
    pub fn new(
        gl_context: &std::rc::Rc<glow::Context>,
        vert_shader_path: &str,
        frag_shader_path: &str,
    ) -> ShaderProgram {
        let vert_shader_src = fs::read_to_string(vert_shader_path).expect(&format!(
            "Failed to read vertex shader at {}",
            vert_shader_path
        ));
        let frag_shader_src = fs::read_to_string(frag_shader_path).expect(&format!(
            "Failed to read fragment shader at {}",
            vert_shader_path
        ));

        let vert_shader_handle = create_shader(gl_context, &vert_shader_src, glow::VERTEX_SHADER);
        let frag_shader_handle =
            create_shader(gl_context, &frag_shader_src, glow::FRAGMENT_SHADER);

        let shader_program_handle =
            create_program(gl_context, vert_shader_handle, frag_shader_handle);

        ShaderProgram {
            handle: shader_program_handle,
        }
    }

    pub fn use_program(&self, gl: &Rc<glow::Context>) {
        unsafe {
            gl.use_program(Some(self.handle));
        }
    }

    fn get_uniform_location(&self, gl: &Rc<glow::Context>, name: &str) -> glow::UniformLocation {
        unsafe {
            match gl.get_uniform_location(self.handle, name) {
                Some(program) => program,
                None => {
                    panic!(
                        "The requested uniform {} is not in the shader {:?}",
                        name, self.handle
                    );
                }
            }
        }
    }

    pub fn set_int(&self, gl: &Rc<glow::Context>, name: &str, value: i32) {
        unsafe {
            gl
                .uniform_1_i32(Some(&self.get_uniform_location(gl,name)), value as i32);
        }
    }

    // GLSL doesn't have bools?
    pub fn set_bool(&self, gl: &Rc<glow::Context>, name: &str, value: bool) {
        self.set_int(gl, name, value as i32);
    }

    pub fn set_float(&self, gl: &Rc<glow::Context>, name: &str, value: f32) {
        unsafe {
            gl
                .uniform_1_f32(Some(&self.get_uniform_location(gl,name)), value);
        }
    }

    pub fn set_vec3(&self, gl: &Rc<glow::Context>, name: &str, value: glm::Vec3) {
        unsafe {
            gl.uniform_3_f32(
                Some( &self.get_uniform_location(gl,name)),
                value.x,
                value.y,
                value.z,
            );
        }
    }

    pub fn set_vec2(&self, gl: &Rc<glow::Context>, name: &str, value: glm::Vec2) {
        unsafe {
            gl
                .uniform_2_f32(Some(&self.get_uniform_location(gl,name)), value.x, value.y);
        }
    }

    pub fn set_mat4(&self, gl: &Rc<glow::Context>, name: &str, value: glm::Mat4) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(
                Some(&self.get_uniform_location(gl,name)),
                false,
                glm::value_ptr(&value).into(),
            );
        }
    }
}

fn create_shader(gl: &Rc<glow::Context>, shader_src: &str, shader_type: u32) -> glow::Shader {
    unsafe {
        let shader_handle = gl.create_shader(shader_type).unwrap();

        gl.shader_source(shader_handle, shader_src);
        gl.compile_shader(shader_handle);

        let mut success = gl.get_shader_compile_status(shader_handle);

        if !success {
            let error_string = gl.get_shader_info_log(shader_handle);

            panic!("{}", error_string);
        } else {
            shader_handle
        }
    }
}

fn create_program(
    gl: &Rc<glow::Context>,
    vert_shader_handle: glow::Shader,
    frag_shader_handle: glow::Shader,
) -> glow::Program {
    unsafe {
        let mut shader_program_handle = gl.create_program().unwrap();
        gl.attach_shader(shader_program_handle, vert_shader_handle);
        gl.attach_shader(shader_program_handle, frag_shader_handle);
        gl.link_program(shader_program_handle);

        let success = gl.get_program_link_status(shader_program_handle);
        if !success {
            let error_string = gl.get_program_info_log(shader_program_handle);
            panic!("{}", error_string);
        } else {
            gl.delete_shader(vert_shader_handle);
            gl.delete_shader(frag_shader_handle);
            shader_program_handle
        }
    }
}
