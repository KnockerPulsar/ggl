use crate::gl::get_gl;
use glow::HasContext;

// u32 -> Which slot the texture uses
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum TextureType {
    Diffuse,
    Specular,
    Emissive,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Texture2D {
    pub handle: glow::Texture,
    pub tex_type: TextureType,
}

impl Texture2D {
    pub fn from_handle(tex_handle: &glow::Texture, tex_type: TextureType) -> Texture2D {
        Texture2D {
            handle: *tex_handle,
            tex_type,
        }
    }

}
