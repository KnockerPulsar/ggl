mod loaders;
mod ui;
mod renderer;

mod camera;
mod ecs;
mod egui_drawable;
mod input;
mod light;
mod light_system;
mod scene;
mod shader;
mod texture;
mod transform;
mod app;
mod gl;

mod model;
mod mesh;

use app::App;
use scene::Scene;
use crate::gl::get_gl;
use crate::input::InputSystem;
use crate::transform::Transform;
use crate::light_system::light_system;

fn main() {
    let (window_width, window_height) = (1280, 720);

    let (mut app, event_loop) = App::init(window_width, window_height);

    let (texture_loader, object_loader, shader_loader) = app.get_resource_managers();
    let  scene = Scene::light_test(window_width as i32, window_height as i32, texture_loader, shader_loader, object_loader);

    app.set_scene(scene);
    
    event_loop.run(move |event, _, control_flow| {
        app.run(event, control_flow);
    });
}
