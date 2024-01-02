use std::rc::Rc;

use crate::{
    loaders::*,
    shader::{ProgramHandle, UniformMap},
    texture::{Texture2D, TextureType},
};

#[allow(dead_code)]
#[derive(Hash, Copy, Clone, PartialEq, Eq)]
pub enum MaterialType {
    Unlit,
    Lit,
    Billboard,
}

#[derive(Hash, Clone, Eq)]
pub struct Material {
    pub shader: ProgramHandle,
    pub material_type: MaterialType,
    pub textures: Vec<Texture2D>,
    pub transparent: bool,
}

impl Material {
    pub fn billboard(shader_loader: &mut ShaderLoader, tex: Texture2D) -> Self {
        Material {
            shader: shader_loader.get_shader_rc(DEFAULT_BILLBOARD_SHADER),
            material_type: MaterialType::Billboard,
            textures: vec![tex],
            transparent: true,
        }
    }

    pub fn lit(shader_loader: &mut ShaderLoader, textures: Vec<Texture2D>) -> Self {
        Material {
            shader: shader_loader.get_shader_rc(DEFAULT_LIT_SHADER),
            material_type: MaterialType::Lit,
            transparent: false,
            textures,
        }
    }

    pub fn set_textures(&mut self, new_tex: Vec<Texture2D>) {
        self.textures = new_tex;
    }

    pub fn default_billboard(
        shader_loader: &mut ShaderLoader,
        texture_loader: &mut TextureLoader,
    ) -> Self {
        let directional_light = texture_loader.directional_light_texture();
        let diffuse_texture = Texture2D::from_native_handle(
            directional_light,
            crate::texture::TextureType::Diffuse,
            1,
        );

        Self::billboard(shader_loader, diffuse_texture)
    }

    pub fn default_unlit(shader_loader: &mut ShaderLoader) -> Self {
        Material {
            shader: shader_loader.get_shader_rc(DEFAULT_UNLIT_SHADER),
            material_type: MaterialType::Unlit,
            textures: vec![],
            transparent: false,
        }
    }

    pub fn default_lit(
        shader_loader: &mut ShaderLoader,
        texture_loader: &mut TextureLoader,
    ) -> Self {
        let checker_diffuse = Texture2D::from_native_handle(
            texture_loader.checker_texture(),
            TextureType::Diffuse,
            1,
        );

        let white_specular =
            Texture2D::from_native_handle(texture_loader.white_texture(), TextureType::Specular, 1);

        Self::lit(shader_loader, vec![checker_diffuse, white_specular])
    }

    pub fn upload_uniforms(&self, uniforms: &UniformMap, prefix: &str) {
        self.shader.upload_uniforms(uniforms, prefix);
    }

    pub fn upload_textures(&self, prefix: &str) {
        self.shader.upload_textures(&self.textures, prefix);
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.shader == other.shader
    }
}
