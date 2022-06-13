use std::rc::Rc;

use glow::HasContext;

use crate::shader::ShaderProgram;

pub struct Texture2D {
    handle: glow::Texture,
}

impl Texture2D {
    pub fn load(gl: &Rc<glow::Context>, path: &str) -> Self {
        let texture = image::io::Reader::open(path).unwrap().decode().unwrap();

        let texture_w = texture.width() as i32;
        let texture_h = texture.height() as i32;

        let texture_handle: glow::Texture;

        unsafe {
            let format = match texture.color() {
                image::ColorType::L8 => glow::RGB,
                image::ColorType::Rgb8 => glow::RGB,
                image::ColorType::Rgba8 => glow::RGBA,
                _ => {
                    panic!("Unsupported color type {:?}", texture.color());
                }
            };

            println!("Loaded texture of format {:#?}", format);
            println!(
                "GL_RED = {:?}, GL_RGB = {:?}, GL_RGBA = {:?}",
                glow::RED,
                glow::RGB,
                glow::RGBA
            );

            texture_handle = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(texture_handle));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                texture_w,
                texture_h,
                0,
                format as u32,
                glow::UNSIGNED_BYTE,
                Some(texture.as_bytes()),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
        }

        Texture2D {
            handle: texture_handle,
        }
    }

    pub fn use_texture(
        &self,
        gl: &Rc<glow::Context>,
        texture_unit_index: u32,
        sampler_name: &str,
        shader: &ShaderProgram,
    ) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + texture_unit_index as u32);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.handle));
            shader.set_int(&gl, sampler_name, texture_unit_index as i32);
        }
    }
}
