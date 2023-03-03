use std::collections::HashMap;
use crate::shader::ShaderProgram;

pub const DEFAULT_SHADER: &'static str= "default";
pub const DEFAULT_BILLBOARD_SHADER: &'static str= "default_billboard";

pub struct ShaderLoader {
    shaders: HashMap<String, ShaderProgram>,
}

impl ShaderLoader {
    pub fn new(custom_shaders: &[(&str, &str, &str)]) -> Self {
        let mut shader_loader = ShaderLoader {
            shaders: HashMap::new(),
        };

        shader_loader.load_shader(DEFAULT_SHADER,"assets/shaders/textured.vert","assets/shaders/lit-textured.frag");
        shader_loader.load_shader(DEFAULT_BILLBOARD_SHADER,"assets/shaders/billboard_textured.vert","assets/shaders/simple.frag");

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
    ) {
        if !self.shaders.contains_key(program_name) {
            println!("Loading shader ({program_name})");

            let shader = ShaderProgram::new(vert_path, frag_path);
            match shader {
                Ok(shader) => { self.shaders.insert(String::from(program_name), shader); } ,
                Err(err) => eprintln!("Failed to load shader: {}", err),
            };
        }
    }

    pub fn borrow_shader(&self, program_name: &str) -> Option<&ShaderProgram> {
        self.shaders.get(program_name)
    }
}
