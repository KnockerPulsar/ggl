use egui::Ui;
pub trait EguiDrawable {
    fn on_egui(&mut self, ui: &mut Ui, index: usize);
}
