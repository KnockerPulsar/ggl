use std::{collections::HashMap, rc::Rc};

use glow::Context;

use crate::texture::{Texture2D, TextureType};

pub struct TextureLoader {
    textures: HashMap<String, Texture2D>,
}

impl TextureLoader {
    pub fn new() -> Self {
        TextureLoader {
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(
        &mut self,
        gl_rc: &Rc<Context>,
        path: &str,
        tex_type: TextureType,
    ) -> (bool, &Texture2D) {
        let path_string = String::from(path);
        let mut first_load = false;

        if !self.textures.contains_key(path) {
            self.textures
                .insert(path_string, Texture2D::load(gl_rc, path, tex_type));

            first_load = true;
        }

        (first_load, &self.textures.get(path).unwrap())
    }
}
