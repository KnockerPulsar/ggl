use crate::ecs::Ecs;
use egui::Ui;

pub trait EguiDrawable {
    fn on_egui(&mut self, ui: &mut Ui, index: usize, ecs: &Ecs) -> bool;
}
