use obj::{ObjError, MtlLibsLoadError};
use crate::enabled_header;
use crate::loaders::DEFAULT_CUBE_NAME;
use crate::mesh::MeshRenderer;
use crate::egui_drawable::EguiDrawable;


#[derive(Hash, Clone)]
pub struct Model {
    pub name: String,
    pub enabled: bool,
    pub mesh_renderers: Vec<MeshRenderer>,
    pub directory: String,
}

impl Model {
    pub fn new(name: impl Into<String>, directory: impl Into<String>, meshes: Vec<MeshRenderer>) -> Self {
        Model {
            name: name.into(),
            directory: directory.into(),
            enabled: true,
            mesh_renderers: meshes
        }
    }

    pub fn add_mesh(&mut self, mr: MeshRenderer) {
        self.mesh_renderers.push(mr);
    }

    #[allow(dead_code)]
    pub fn get_mesh_renderer(&self, index: usize) -> &MeshRenderer {
        &self.mesh_renderers[index]
    }
}

impl EguiDrawable for Model {
    fn on_egui(&mut self, ui: &mut egui::Ui, index: usize) -> bool {
        enabled_header!(self, ui, "Model", index, {
            ui.horizontal(|ui| {
                ui.label(format!("Name: {}", self.name));

                if ui.button("Load model").clicked() {
                    let path = rfd::FileDialog::new().add_filter("Object model", &["obj"]).pick_file(); 
                    if let Some(path) = path { 
                        let t = path.to_str().unwrap_or(DEFAULT_CUBE_NAME).to_owned();
                        self.name = t;
                    }
                }
            });

            ui.vertical(|_ui| {

            });
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
