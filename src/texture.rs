
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[allow(dead_code)]
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
