use glow::HasContext;
use std::collections::HashMap;
use crate::get_gl;

pub struct TextureLoader {
    textures: HashMap<String, glow::Texture>,
}

impl TextureLoader {
    pub fn new(default_textures: &[&str]) -> Self {
        let mut texture_loader = TextureLoader {
            textures: HashMap::new(),
        };

        println!(
            "GL_RED = {:?}, GL_RGB = {:?}, GL_RGBA = {:?}",
            glow::RED,
            glow::RGB,
            glow::RGBA
        );

        let (w, h) = (1i32, 1i32);
        let pixel = vec![255u8, 0u8, 255u8];
        let buffer: Vec<u8> = pixel
            .iter()
            .cycle()
            .take(pixel.len() * w as usize * h as usize)
            .map(|x| *x)
            .collect();

        assert!( buffer.len() == w as usize * h as usize * 3 );

        texture_loader
            .textures
            .insert(
                "default".to_owned(), 
                Self::from_data(
                    (w, h), 
                    glow::RGB, 
                    &buffer
                )
            );

        for path in default_textures {
            texture_loader.load_texture(path);
        }

        texture_loader
    }

    pub fn from_data(
        (texture_w, texture_h): (i32, i32), 
        format: u32, 
        texture_data: &[u8], 
    ) -> glow::Texture {
        unsafe {
            let gl = get_gl();
            let texture_handle = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(texture_handle));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                texture_w,
                texture_h,
                0,
                format,
                glow::UNSIGNED_BYTE,
                Some(texture_data),
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

            texture_handle
        }
    }

    pub fn load_into_handle(&self,  path: &str) -> Option<glow::Texture> {
        let texture = match image::io::Reader::open(path) {
            Ok(encoded_texture) => encoded_texture.decode().unwrap(),
            Err(_) => return None,
        };

        let texture_w = texture.width() as i32;
        let texture_h = texture.height() as i32;

        let format = match texture.color() {
            image::ColorType::L8 => glow::RED,
            image::ColorType::Rgb8 => glow::RGB,
            image::ColorType::Rgba8 => glow::RGBA,
            _ => {
                panic!("Unsupported color type {:?}", texture.color());
            }
        };

        println!("Loaded texture [{}] of format {:#?}", path, format);


        Some(Self::from_data(
            (texture_w, texture_h), 
            format,
            texture.as_bytes()
        ))
    }

    pub fn load_texture(&mut self, path: &str) -> &glow::Texture {
        let path_string = String::from(path);

        if !self.textures.contains_key(path) {
            if let Some(texture_handle) = self.load_into_handle(path) {
                self.textures.insert(path_string, texture_handle);
            } 
        }

        match self.textures.get(path) {
            Some(tex) => tex,
            None => { 
                println!("Failed to load texture at {path}, returning default texture");
                self.textures.get("default").unwrap()
            }
        }
    }
}
