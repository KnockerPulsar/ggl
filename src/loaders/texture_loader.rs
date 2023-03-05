use glow::HasContext;
use std::{collections::HashMap, path::Path};
use crate::get_gl;

const DEFAULT_TEXTURE: &'static str = "default";
const DEFAULT_TEXTURES: [&str; 6] = [
    "assets/textures/white.jpeg",
    "assets/textures/black.jpg",
    "assets/textures/grid.jpg",
    "assets/textures/checker_32_32.jpg",
    "assets/textures/point_light_white.png",
    "assets/textures/directional_light_white.png"
];


macro_rules! default_texture_getters {
    ($( ($name: expr, $fn_name: tt) ),*) => {
        $(
            #[allow(dead_code)]
            pub fn $fn_name(&self) -> glow::Texture {
                self.borrow($name)
            }
        )*
    };
}

pub struct TextureLoader {
    /// Maps from the texture's name to its native handle
    /// Note that the native handle does not specify the type of this texture (Diffuse, Specular,
    /// Emissive, etc...)
    textures: HashMap<String, glow::Texture>,
}

impl TextureLoader {
    pub fn new() -> Self {
        let mut texture_loader = TextureLoader {
            textures: HashMap::new(),
        };
        
        texture_loader.setup_default_texture();

        for path in DEFAULT_TEXTURES {
            texture_loader.load_texture(Path::new(path));
        }

        texture_loader
    }

    fn setup_default_texture(&mut self) {
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
            .take(pixel.len() * w as usize * h as usize).copied()
            .collect();

        assert!( buffer.len() == w as usize * h as usize * 3 );

        self.textures
            .insert(
                DEFAULT_TEXTURE.to_owned(), 
                Self::from_data(
                    (w, h), 
                    glow::RGB, 
                    &buffer
                )
            );
    }

    fn from_data(
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

    /// Usually used when loading a texture for the first time.
    /// This mostly occurs when loading a mesh for example.
    fn load_into_handle(&self, path: &Path) -> glow::Texture {
        let path_string = path.to_str().unwrap();
        let texture = image::io::Reader::open(path_string).unwrap().decode().unwrap();

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

        // TODO: Find a way to make this const or static (no need to construct it every time we
        // call this function)
        let int_to_texture_format: HashMap<u32, &'static str> = HashMap::from([
            (glow::RED, "GL_RED"),
            (glow::RGB, "GL_RGB"),
            (glow::RGBA, "GL_RGBA")
        ]);

        println!("Loaded texture [{}] of format {:#?}", path_string, int_to_texture_format.get(&format).unwrap());


        Self::from_data(
            (texture_w, texture_h), 
            format,
            texture.as_bytes()
        )
    }

    /// Used when we want to make sure a texture is loaded at most one time.
    /// If the user attempt to load it again, it will return the already loaded instance.
    /// Returns a fallback/default texture if the given texture path does not exist.
    pub fn load_texture(&mut self, path: &Path) -> glow::Texture {
        if path.exists() && path.is_file() {
            let file_name = path.file_stem().unwrap().to_str().unwrap();

            if self.textures.contains_key(file_name) {
                return *self.textures.get(file_name).unwrap()
            }

            let texture_handle = self.load_into_handle(path);
            self.textures.insert(file_name.to_string(), texture_handle);

            texture_handle
        } else {
            let texture_path = path.to_str().unwrap();
            println!("Failed to load texture at {texture_path}, returning error texture");
            self.default_texture()
        }
    }

    /// Borrows a texture given its name.
    /// The name is the file name WITHOUT its extension, for example: `path/to/img.png`.
    /// Here `img` is the string that should be passed to the function.
    pub fn borrow(&self, file_name: &str) -> glow::Texture {
        match self.textures.get(file_name) {
            Some(tex) => *tex,
            None => { 
                println!("Failed to load the texture named {file_name}, returning error texture");
                self.default_texture()
            }
        }
    }

    default_texture_getters![ 
        (DEFAULT_TEXTURE, default_texture),
        ("checker_32_32", checker_texture),
        ("white", white_texture),
        ("black", black_texture),
        ("point_light_white", point_light_texture),
        ("directional_light_white", directional_light_texture)
    ];
}
