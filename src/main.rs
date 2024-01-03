mod loaders;
mod renderer;
mod ui;

mod app;
mod camera;
mod ecs;
mod egui_drawable;
mod gl;
mod input;
mod light;
mod light_system;
mod scene;
mod shader;
mod texture;
mod transform;

mod mesh;
mod model;

use crate::gl::get_gl;
use crate::input::InputSystem;
use crate::light_system::light_system;
use crate::transform::Transform;
use app::App;
use scene::Scene;

fn main() {
    let (window_width, window_height) = (1280, 720);

    let (mut app, event_loop) = App::init(window_width, window_height);

    let (texture_loader, object_loader, shader_loader) = app.get_resource_managers();
    let scene = Scene::light_test(
        window_width as i32,
        window_height as i32,
        texture_loader,
        shader_loader,
        object_loader,
    );

    app.set_scene(scene);

    event_loop.run(move |event, _, control_flow| {
        app.run(event, control_flow);
    });
}
