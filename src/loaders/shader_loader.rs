use crate::{
    map,
    shader::{ProgramHandle, UniformMap},
};
use std::{collections::HashMap, rc::Rc};

pub const DEFAULT_UNLIT_SHADER: &str = "default_unlit";
pub const DEFAULT_BILLBOARD_SHADER: &str = "default_billboard";
pub const DEFAULT_LIT_SHADER: &str = "default_lit";

const DEFAULT_SHADERS: [(&str, &str, &str); 3] = [
    (
        DEFAULT_UNLIT_SHADER,
        "assets/shaders/default_unlit.vert",
        "assets/shaders/default_unlit.frag",
    ),
    (
        DEFAULT_BILLBOARD_SHADER,
        "assets/shaders/billboard_textured.vert",
        "assets/shaders/simple.frag",
    ),
    (
        DEFAULT_LIT_SHADER,
        "assets/shaders/textured.vert",
        "assets/shaders/lit-textured.frag",
    ),
];

pub struct ShaderLoader {
    shaders: HashMap<String, ProgramHandle>,
}

impl ShaderLoader {
    pub fn new(custom_shaders: &[(&str, &str, &str, UniformMap)]) -> Self {
        let mut shader_loader = ShaderLoader {
            shaders: HashMap::new(),
        };

        for (program_name, vert_path, frag_path, uniforms) in custom_shaders {
            shader_loader.load_shader(program_name, vert_path, frag_path, uniforms.clone());
        }

        for (program_name, vert_path, frag_path) in DEFAULT_SHADERS {
            shader_loader.load_shader(program_name, vert_path, frag_path, map! {});
        }

        shader_loader
    }

    pub fn load_shader(
        &mut self,
        program_name: &str,
        vert_path: &str,
        frag_path: &str,
        uniforms: UniformMap,
    ) {
        if !self.shaders.contains_key(program_name) {
            println!("Loading shader ({program_name})");

            let shader = ProgramHandle::new(vert_path, frag_path, uniforms);
            match shader {
                Ok(shader) => {
                    self.shaders.insert(String::from(program_name), shader);
                }
                Err(err) => eprintln!("Failed to load shader: {}", err),
            };
        }
    }

    pub fn get_shader_rc(&mut self, program_name: &str) -> ProgramHandle {
        let shader = if self.shaders.contains_key(program_name) {
            self.shaders.get_mut(program_name)
        } else {
            self.shaders.get_mut(DEFAULT_UNLIT_SHADER)
        };

        shader.unwrap().clone()
    }
}
