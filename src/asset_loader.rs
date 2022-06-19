use glow::{Context, HasContext};
use std::{collections::HashMap, rc::Rc};

pub struct TextureLoader {
    textures: HashMap<String, glow::Texture>,
}

impl TextureLoader {
    pub fn new() -> Self {
        TextureLoader {
            textures: HashMap::new(),
        }
    }

    pub fn load_into_handle(&self, gl: &Rc<Context>, path: &str) -> glow::Texture {
        let texture = image::io::Reader::open(path).unwrap().decode().unwrap();

        let texture_w = texture.width() as i32;
        let texture_h = texture.height() as i32;

        let texture_handle: glow::Texture;

        unsafe {
            let format = match texture.color() {
                image::ColorType::L8 => glow::RED,
                image::ColorType::Rgb8 => glow::RGB,
                image::ColorType::Rgba8 => glow::RGBA,
                _ => {
                    panic!("Unsupported color type {:?}", texture.color());
                }
            };

            if self.textures.len() == 0 {
                println!(
                    "GL_RED = {:?}, GL_RGB = {:?}, GL_RGBA = {:?}",
                    glow::RED,
                    glow::RGB,
                    glow::RGBA
                );
            }

            println!("Loaded texture [{}] of format {:#?}", path, format,);

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

        texture_handle
    }

    pub fn load_texture(&mut self, gl_rc: &Rc<Context>, path: &str) -> (bool, &glow::Texture) {
        let path_string = String::from(path);
        let mut first_load = false;

        if !self.textures.contains_key(path) {
            self.textures
                .insert(path_string, self.load_into_handle(gl_rc, path));

            first_load = true;
        }

        (first_load, self.textures.get(path).unwrap())
    }
}
