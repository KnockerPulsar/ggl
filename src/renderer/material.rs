use crate::{
    loaders::*,
    texture::{Texture2D, TextureType}, 
    shader::UniformMap, 
};

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MaterialType {
    Unlit,
    Lit,
    Billboard
}

#[derive(Clone, Eq)]
pub struct Material {
    pub shader_ref: &'static str,
    pub material_type: MaterialType,
    pub textures: Vec<Texture2D>,
    pub transparent: bool
}

impl Material {
    pub fn default_billboard(texture_loader: &mut TextureLoader) -> Self {
        let directional_light = texture_loader.directional_light_texture();
        let diffuse_texture = Texture2D::from_native_handle(
            directional_light,
            crate::texture::TextureType::Diffuse,
            1
        );

        Material {
            shader_ref   : DEFAULT_BILLBOARD_SHADER,
            material_type: MaterialType::Billboard,
            textures     : vec![diffuse_texture],
            transparent  : true
        }
    }

    pub fn default_unlit(_shader_loader: &mut ShaderLoader) -> Self {
        Material {
            shader_ref   : DEFAULT_UNLIT_SHADER,
            material_type: MaterialType::Unlit,
            textures     : vec![],
            transparent  : false
        }
    }

    pub fn default_lit(texture_loader: &mut TextureLoader) -> Self {
        let checker_diffuse = Texture2D::from_native_handle(
            texture_loader.checker_texture(),
            TextureType::Diffuse,
            1
        );

        let white_specular = Texture2D::from_native_handle(
            texture_loader.white_texture(),
            TextureType::Specular,
            1
        );

        Material {
            shader_ref   : DEFAULT_LIT_SHADER,
            material_type: MaterialType::Lit,
            textures     : vec![checker_diffuse, white_specular],
            transparent  : false
        }
    }

    pub fn shader_ref(&self) -> &'static str {
        self.shader_ref
    }

    pub fn upload_uniforms(&self, shader_loader: &mut ShaderLoader, uniforms: &UniformMap, prefix: &str) {
        shader_loader
            .borrow_shader(self.shader_ref())
            .upload_uniforms(uniforms, prefix);
    }

    pub(crate) fn upload_textures(&self, shader_loader: &mut ShaderLoader, prefix: &str) {
        let shader = shader_loader.borrow_shader(self.shader_ref);
        shader.upload_textures(&self.textures, prefix);
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.shader_ref() == other.shader_ref()
    }
}
