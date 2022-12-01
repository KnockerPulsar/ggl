use std::collections::HashMap;
use crate::shader::ShaderProgram;

pub struct ShaderLoader {
    shaders: HashMap<String, ShaderProgram>,
}

impl ShaderLoader {
    pub fn new(custom_shaders: &[(&str, &str, &str)]) -> Self {
        let mut shader_loader = ShaderLoader {
            shaders: HashMap::new(),
        };

        shader_loader.load_shader("default","assets/shaders/textured.vert","assets/shaders/lit-textured.frag");

        for (program_name, vert_path, frag_path) in custom_shaders {
            shader_loader.load_shader(program_name, vert_path, frag_path);
        }

        shader_loader
    }

    pub fn load_shader(
        &mut self,
        program_name: &str,
        vert_path: &str,
        frag_path: &str,
    ) -> &ShaderProgram {
        if !self.shaders.contains_key(program_name) {
            self.shaders.insert(
                String::from(program_name),
                ShaderProgram::new(vert_path, frag_path),
            );
        }

        self.shaders.get(program_name).unwrap()
    }

    pub fn borrow_shader(&self, program_name: &str) -> Option<&ShaderProgram> {
        self.shaders.get(program_name)
    }
}
