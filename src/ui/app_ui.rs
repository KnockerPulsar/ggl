use std::path::PathBuf;

use crate::{
    scene::Scene,
    input::InputSystem,
    ecs::{Ecs, ADDABLE_COMPONENTS},
    transform::Transform,
    loaders::ObjLoader,
};
use egui::{Ui, Context, LayerId};
use egui_gizmo::{GizmoMode, Gizmo, GizmoOrientation};


pub fn draw_gizmo(ui: &mut Ui, scene: &Scene) {
    let Some(eid) = scene.selected_entity else { return };

    scene.ecs.do_entity(eid, |selected_entity_transform: &mut Transform| {
        let gizmo = Gizmo::new("My gizmo")
            .view_matrix(scene.camera.get_view_matrix())
            .projection_matrix(scene.camera.get_proj_matrix())
            .model_matrix(selected_entity_transform.get_model_matrix())
            .mode(scene.gizmo_mode)
            .orientation(GizmoOrientation::Local);

        if let Some(response) = gizmo.interact(ui) {
            selected_entity_transform.set_model(response.transform.into());
        }
    });
}

pub fn selected_entity_gizmo(ctx: &Context, current_scene: &mut Scene, input: &InputSystem) {
    let area = egui::Area::new("Gizmo");

    area.show(ctx, |ui| {
        // Needed for the gizmo to respond to inputs
        ui.with_layer_id(LayerId::background(), |ui| {
            draw_gizmo(ui, current_scene);
        })
    });

    if input.is_down(glutin::event::VirtualKeyCode::T) {
        current_scene.gizmo_mode = GizmoMode::Translate;
    }

    if input.is_down(glutin::event::VirtualKeyCode::R) {
        current_scene.gizmo_mode = GizmoMode::Rotate;
    }

    if input.is_down(glutin::event::VirtualKeyCode::Y) {
        current_scene.gizmo_mode = GizmoMode::Scale;
    }
}

fn add_component_ui(ui: &mut Ui, ecs: &mut Ecs) -> bool {

    ui.horizontal(|ui| {

        let combobox = egui::ComboBox::from_label("")
            .selected_text(ecs.component_to_add.to_string());

        // Changed component to be potentially added
        combobox.show_ui(ui, |ui| {
            ADDABLE_COMPONENTS.map(|comp| {
                ui.selectable_value(&mut ecs.component_to_add, comp, comp.to_string());
            })
        });

        // Add the currently seleccted component type
        ui.button("Add component").clicked()
    }).inner
}


// Shows the entity's name and a list of its components.
fn selected_entity_ui(ui: &mut Ui, ecs: &mut Ecs, selection: usize) -> bool {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().outer_margin(10.0))
        .show_inside(ui, |ui| {
            let name = ecs.do_entity(selection, |t: &mut Transform| { t.get_name().to_string() });
            ui.vertical_centered(|ui| { ui.label(egui::RichText::new(name).heading().strong()); });

            ui.add_space(10.);

            ecs.component_vecs
                .iter_mut()
                .for_each(|cv| cv.as_mut().draw_egui(ui, selection));

            ui.add_space(10.);
            ui.separator();
            ui.add_space(10.);

            add_component_ui(ui, ecs)
        }).inner
}

/// Shows a list of buttons where the label is the entity's name.
/// Returns Some(id) if an entity was clicked. None otherwise.
fn entity_selection(ui: &mut Ui, ecs: &mut Ecs) -> (Option<usize>, bool) {
    let layout = egui::Layout::from_main_dir_and_cross_align(egui::Direction::TopDown, egui::Align::Center).with_cross_justify(true);
   
    egui::SidePanel::left("Entities")
        .resizable(true)
        .width_range(100.0..=200.0)
        .default_width(120.0)
        .show_inside(ui, |ui| {
            ui.heading("Entities");
            ui.with_layout(layout, |ui| {
                let clicked = ecs.do_all_some(|(id, transform): (usize, &mut Transform)| {
                    ui.button(transform.get_name())
                        .clicked()
                        .then_some(id) // Some(id) iff clicked            
                });

                ui.separator();

                let new_entity = ui.button("New Empty Entity").clicked();

                // Shouldn't click more than one button
                (clicked.first().copied(), new_entity)
            }).inner
        }).inner
}

pub fn entities_panel(
    ui: &mut Ui,
    scene: &mut Scene,
    lights_on: &mut bool
) -> (Option<usize>, bool, bool) {
    // ui.spacing();

    let (selected_entity, add_entity, add_component) = {
        let (selected_entity, add_entity) = entity_selection(ui, &mut scene.ecs);
        let selected_entity = selected_entity.or(scene.selected_entity);

        let add_component = 
            if let Some(selection) = selected_entity {
                selected_entity_ui(ui, &mut scene.ecs, selection)
            } else {
                false
            };

        (selected_entity, add_entity, add_component)
    };

    // ui.checkbox(lights_on, "Global light toggle");
    (selected_entity, add_entity, add_component)
}

pub fn models_panel(ui: &mut Ui, object_loader: &mut ObjLoader) -> Option<PathBuf> {
    ui.vertical_centered(|ui| {
        ui.heading("Loaded Models");
        ui.separator();

        object_loader.models().iter().for_each(|(name, _)| {
            ui.label(name.to_string());
        });

        ui.add_space(20.0);

        let load_a_model = egui::RichText::new("Load a model").size(20.0).strong();
        if ui.button(load_a_model).clicked() {
            rfd::FileDialog::new().add_filter("Object model", &["obj"]).pick_file()
        } else {
            None
        }
    }).inner
}
