use glow::HasContext;
use crate::gl::get_gl;


#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TextureType {
    Diffuse,
    Specular,
    Emissive,
}

/// I follow the following naming convention in shaders:
///     uniform sampler2D {uniform_name}{tex_index};
///
/// So for a texture with a name `texture_diffuse` and and index of `1`:
///     uniform sampler2D texture_diffuse1;
///
/// If you have an array of textures [ diffuse, diffuse, specular, emissive ], they'll be 
/// uploaded as follows:
/// diffuse  => texture_diffuse1 , unit 0
/// diffuse  => texture_diffuse2 , unit 1
/// specular => texture_sepcular1, unit 2
/// emissive => texture_emissive1, unit 3
#[derive(Copy, Clone, Debug, Eq)]
pub struct Texture2D {
    pub native_handle: glow::Texture,
    pub tex_type: TextureType,
    pub tex_index: u32
}

impl Texture2D {
    pub fn from_native_handle(
        tex_handle: glow::Texture, 
        tex_type: TextureType, 
        tex_index: u32,
    ) -> Texture2D {
        // assert!( tex_index > 0, "Texture indices must be > 1 (Or change them in the shader to start from 0)" );
        Texture2D {
            native_handle: tex_handle,
            tex_type,
            tex_index
        }
    }

    pub fn activate_and_bind(&self, unit: u32) {
        unsafe { 
            get_gl().active_texture(glow::TEXTURE0 + unit);
            get_gl().bind_texture(glow::TEXTURE_2D, Some(self.native_handle)); 
        }
    }
}

impl PartialEq for Texture2D {
    fn eq(&self, other: &Self) -> bool {
        self.native_handle == other.native_handle && self.tex_type == other.tex_type
    }
}
