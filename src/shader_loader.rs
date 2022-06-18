use std::collections::HashMap;
use std::rc::Rc;

use crate::shader::ShaderProgram;
use glow::Context;

pub struct ShaderLoader {
    shaders: HashMap<String, ShaderProgram>,
}

impl ShaderLoader {
    pub fn new() -> Self {
        ShaderLoader {
            shaders: HashMap::new(),
        }
    }

    pub fn load_shader(
        &mut self,
        gl_rc: &Rc<Context>,
        program_name: &str,
        vert_path: &str,
        frag_path: &str,
    ) -> &ShaderProgram {
        if !self.shaders.contains_key(program_name) {
            self.shaders.insert(
                String::from(program_name),
                ShaderProgram::new(gl_rc, vert_path, frag_path),
            );
        }

        self.shaders.get(program_name).unwrap()
    }

    pub fn borrow_shader(&self, program_name: &str) -> Option<&ShaderProgram> {
        self.shaders.get(program_name)
    }
}
