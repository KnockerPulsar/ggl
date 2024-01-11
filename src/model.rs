use crate::enabled_header;
use crate::gl::get_gl;
use crate::loaders::DEFAULT_CUBE_NAME;
use crate::mesh::Mesh;
use crate::renderer::Material;
use crate::shader::UniformMap;
use crate::{ecs::Ecs, egui_drawable::EguiDrawable};
use glow::HasContext;
use obj::{MtlLibsLoadError, ObjError};

#[derive(Clone, Default)]
pub struct Model {
    pub name: String,
    pub directory: String,

    pub enabled: bool,
    pub meshes: Vec<Mesh>,
    pub material: Option<Material>,
}

impl Model {
    pub fn new(
        name: impl Into<String>,
        directory: impl Into<String>,
        meshes: Vec<Mesh>,
        material: Option<Material>,
    ) -> Self {
        Model {
            name: name.into(),
            directory: directory.into(),
            enabled: true,
            meshes,
            material,
        }
    }

    pub fn draw(&self, external_uniforms: &UniformMap) {
        let material = self
            .material
            .as_ref()
            .expect("All models need to have a matieral!");

        material.shader.use_program(); // TODO: Bind once per material group
        material.upload_uniforms();
        material.upload_external_uniforms(external_uniforms);

        unsafe {
            let gl_rc = get_gl();
            self.meshes.iter().for_each(|mesh| {
                gl_rc.bind_vertex_array(Some(mesh.vao()));
                gl_rc.draw_elements(
                    glow::TRIANGLES,
                    mesh.indices.len() as i32,
                    glow::UNSIGNED_INT,
                    0,
                );
            })
        }
    }
}

impl EguiDrawable for Model {
    fn on_egui(&mut self, ui: &mut egui::Ui, index: usize, ecs: &Ecs) -> bool {
        enabled_header!(self.enabled, ui, "Model", index, {
            ui.horizontal(|ui| {
                ui.label(format!("Name: {}", self.name));

                if ui.button("Load model").clicked() {
                    let path = rfd::FileDialog::new()
                        .add_filter("Object model", &["obj"])
                        .pick_file();
                    if let Some(path) = path {
                        let t = path.to_str().unwrap_or(DEFAULT_CUBE_NAME).to_owned();
                        self.name = t;
                    }
                }
            });

            if let Some(material) = &mut self.material {
                material.material_kind.on_egui(ui, index, ecs);
            }
        });
        false
    }
}

#[derive(Debug)]
pub enum ObjLoadError {
    Obj(ObjError),
    Mtl(MtlLibsLoadError),
}

impl From<ObjError> for ObjLoadError {
    fn from(value: ObjError) -> Self {
        ObjLoadError::Obj(value)
    }
}

impl From<MtlLibsLoadError> for ObjLoadError {
    fn from(value: MtlLibsLoadError) -> Self {
        ObjLoadError::Mtl(value)
    }
}
