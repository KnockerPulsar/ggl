use nalgebra_glm::{vec3, Vec3};

use crate::{
    egui_drawable::EguiDrawable,
    light::float3_slider,
    loaders::*,
    shader::{ProgramHandle, UniformMap},
    texture::{activate_and_bind, Texture, TextureType},
};

const MIN_SHININESS: f32 = 2.0f32;

#[derive(Clone, Copy, PartialEq)]
pub struct LitUniforms {
    pub diffuse: [Option<glow::Texture>; 3],
    pub specular: [Option<glow::Texture>; 3],
    pub emissive: Option<glow::Texture>,
    pub emissive_factor: Vec3,
    pub shininess: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub struct BillboardUniforms {
    pub diffuse: glow::Texture,
}

#[derive(Copy, Clone, PartialEq)]
pub enum MaterialKind {
    Unlit,
    Lit(LitUniforms),
    Billboard(BillboardUniforms),
}

impl EguiDrawable for MaterialKind {
    fn on_egui(&mut self, ui: &mut egui::Ui, _index: usize, _ecs: &crate::ecs::Ecs) -> bool {
        match self {
            MaterialKind::Unlit => {
                ui.label("Unlit material");
                false
            }
            MaterialKind::Lit(lit_uniforms) => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("emissive_factor");
                        ui.label("shininess");
                    });

                    ui.vertical(|ui| {
                        float3_slider(&mut lit_uniforms.emissive_factor, ui)
                            || ui
                                .add(egui::DragValue::new(&mut lit_uniforms.shininess).speed(0.01))
                                .changed()
                    })
                    .inner
                })
                .inner
            }
            MaterialKind::Billboard(_) => {
                ui.label("Billboard material");
                false
            }
        }
    }
}

impl MaterialKind {
    fn upload_uniforms(&self, shader: &ProgramHandle) {
        match self {
            MaterialKind::Unlit => {}
            MaterialKind::Lit(lit_uniforms) => {
                let mut counter = 1;

                let mut upload_textures = |tex_array: [Option<glow::Texture>; 3], prefix| {
                    tex_array.iter().enumerate().for_each(|(index, tex)| {
                        let Some(tex) = tex else {
                            return;
                        };

                        activate_and_bind(tex, counter);
                        shader.set_int(&format!("{}{}", prefix, index + 1), counter as i32);

                        counter += 1;
                    });
                };

                upload_textures(lit_uniforms.diffuse, "u_material.texture_diffuse");
                upload_textures(lit_uniforms.specular, "u_material.texture_specular");

                if let Some(tex) = lit_uniforms.emissive {
                    activate_and_bind(&tex, counter);
                    shader.set_int("u_material.texture_emissive1", counter as i32);
                }

                shader.set_vec3("u_material.emissive_factor", lit_uniforms.emissive_factor);
                shader.set_float("u_material.shininess", lit_uniforms.shininess);
            }

            MaterialKind::Billboard(billboard_uniforms) => {
                activate_and_bind(&billboard_uniforms.diffuse, 0);
                shader.set_int("texture_diffuse1", 0);
            }
        }
    }
}

#[derive(Clone)]
pub struct Material {
    pub shader: ProgramHandle,
    pub transparent: bool,

    pub material_kind: MaterialKind,
}

impl Material {
    pub fn billboard(
        shader_loader: &mut ShaderLoader,
        billboard_uniforms: BillboardUniforms,
    ) -> Self {
        Material {
            shader: shader_loader.get_shader(DEFAULT_BILLBOARD_SHADER),
            transparent: true,

            material_kind: MaterialKind::Billboard(billboard_uniforms),
        }
    }

    pub fn lit(shader_loader: &mut ShaderLoader, mut lit_uniforms: LitUniforms) -> Self {
        if lit_uniforms.shininess < MIN_SHININESS {
            eprintln!(
                "Shininess < {min_shininess} ({}), will set to {min_shininess}",
                lit_uniforms.shininess,
                min_shininess = MIN_SHININESS,
            );
            lit_uniforms.shininess = 2.0;
        }

        Material {
            shader: shader_loader.get_shader(DEFAULT_LIT_SHADER),
            transparent: false,

            material_kind: MaterialKind::Lit(lit_uniforms),
        }
    }

    pub fn default_billboard(
        shader_loader: &mut ShaderLoader,
        texture_loader: &mut TextureLoader,
    ) -> Self {
        Self::billboard(
            shader_loader,
            BillboardUniforms {
                diffuse: texture_loader.directional_light_texture(),
            },
        )
    }

    pub fn default_unlit(shader_loader: &mut ShaderLoader) -> Self {
        Material {
            shader: shader_loader.get_shader(DEFAULT_UNLIT_SHADER),
            transparent: false,

            material_kind: MaterialKind::Unlit,
        }
    }

    pub fn default_lit(
        shader_loader: &mut ShaderLoader,
        texture_loader: &mut TextureLoader,
    ) -> Self {
        Self::lit(
            shader_loader,
            LitUniforms {
                diffuse: [Some(texture_loader.checker_texture()), None, None],
                specular: [Some(texture_loader.white_texture()), None, None],
                emissive: None,
                emissive_factor: vec3(0., 0., 0.),
                shininess: 32.0,
            },
        )
    }

    pub fn from_textures(shader_loader: &mut ShaderLoader, textures: &Vec<Texture>) -> Self {
        let diff = textures
            .iter()
            .filter(|t| t.kind == TextureType::Diffuse)
            .map(|t| t.texture)
            .map(Option::Some)
            .collect::<Vec<_>>();

        assert!(diff.len() <= 3);

        let spec = textures
            .iter()
            .filter(|t| t.kind == TextureType::Specular)
            .map(|t| t.texture)
            .map(Option::Some)
            .collect::<Vec<_>>();

        assert!(spec.len() <= 3);

        let mut diffuse = [None; 3];
        diffuse
            .iter_mut()
            .zip(diff.iter())
            .for_each(|(dest, source)| *dest = *source);

        let mut specular = [None; 3];
        specular
            .iter_mut()
            .zip(diff.iter())
            .for_each(|(dest, source)| *dest = *source);

        Material {
            shader: shader_loader.get_shader(DEFAULT_LIT_SHADER),
            transparent: false,
            material_kind: MaterialKind::Lit(LitUniforms {
                diffuse,
                specular,
                emissive: None,
                emissive_factor: vec3(0., 0., 0.),
                shininess: MIN_SHININESS,
            }),
        }
    }

    pub fn upload_uniforms(&self) {
        self.material_kind.upload_uniforms(&self.shader);
    }

    pub fn upload_external_uniforms(&self, external_uniforms: &UniformMap) {
        external_uniforms
            .iter()
            .for_each(|(uniform_name, uniform_value)| {
                uniform_value.upload(uniform_name, &self.shader)
            })
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.shader == other.shader
    }
}
