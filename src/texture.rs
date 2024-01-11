use crate::gl::get_gl;
use glow::HasContext;

pub fn activate_and_bind(texture: &glow::Texture, unit: u32) {
    unsafe {
        get_gl().active_texture(glow::TEXTURE0 + unit);
        get_gl().bind_texture(glow::TEXTURE_2D, Some(*texture));
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TextureType {
    Diffuse,
    Specular,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Texture {
    pub texture: glow::Texture,
    pub kind: TextureType,
}
