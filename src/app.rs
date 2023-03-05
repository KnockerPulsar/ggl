use std::{sync::Arc, env};

use glow::HasContext;
use glutin::{event::*, event_loop::ControlFlow};
use crate::{
    add_component, 
    ui::*, 
    light::*,
    ecs::AddableComponent, renderer::Renderer
};

use crate::{
    loaders::*,
    gl::get_gl,
    scene::Scene,
    input::InputSystem,
    transform::Transform,
};

pub type EventLoop = glutin::event_loop::EventLoop<()>;

#[derive(Default, PartialEq)]
enum Panels {
    #[default]
    Entities,
    Models,
}

pub struct App {
    glow: egui_glow::EguiGlow,
    renderer: Renderer,
    current_scene: Scene, 

    shader_loader : ShaderLoader,
    texture_loader: TextureLoader,
    object_loader : ObjLoader,
    input: InputSystem,

    last_frame: std::time::Instant,
    current_panel: Panels
}

impl App {
    pub fn init(
        window_width: usize, 
        window_height: usize,
    ) -> (App, glutin::event_loop::EventLoop<()>) {

        let event_loop: EventLoop = glutin::event_loop::EventLoopBuilder::with_user_event().build();
        let renderer = Renderer::new(window_width, window_height, &event_loop);
        let egui_glow = egui_glow::EguiGlow::new(&event_loop, Arc::clone(get_gl()));

        println!(
            "Current working directory: {}",
            env::current_dir().unwrap().to_str().unwrap()
        );

        println!("Loading, please wait...");

        let custom_shaders = [
            ("lit-textured",
             "assets/shaders/textured.vert",
             "assets/shaders/lit-textured.frag"),
        ];

        let mut shader_loader = ShaderLoader::new(&custom_shaders);
        let mut texture_loader = TextureLoader::new();
        let object_loader = ObjLoader::new(&mut shader_loader, &mut texture_loader);


        let last_frame = std::time::Instant::now();
        let input = InputSystem::new();


        let empty_scene = Scene::empty(window_width as i32, window_height as i32);
        let app = App {
            renderer,
            glow: egui_glow,

            current_scene: empty_scene,

            shader_loader,
            texture_loader,
            object_loader,       
            input,

            last_frame,
            current_panel: Panels::Entities
        };

        (app, event_loop)
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.current_scene = scene
    }

    fn app_ui(&mut self) {
        self.glow.run(self.renderer.window.window(), |egui_ctx| {
            egui::Window::new("ggl").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.current_panel, Panels::Entities, "Entities");
                    ui.selectable_value(&mut self.current_panel, Panels::Models, "Models");
                });

                selected_entity_gizmo(egui_ctx, &mut self.current_scene, &self.input);

                match self.current_panel {
                    Panels::Entities => {
                        let (selected_entity, add_entity, add_component) = entities_panel(
                            ui,
                            &mut self.current_scene,
                            &mut self.renderer.lights_on
                        );

                        self.current_scene.selected_entity = selected_entity;

                        if add_entity {
                            self.current_scene.ecs.add_empty_entity();
                        }

                        if let Some(selected_entity) = self.current_scene.selected_entity {
                            if add_component {
                                add_component!(
                                    &mut self.current_scene.ecs,
                                    selected_entity,
                                    [Transform, PointLight, SpotLight, DirectionalLight]
                                );
                            }
                        }
                    }, 
                    Panels::Models => {
                        let path = models_panel(ui, &mut self.object_loader);
                        
                        let Some(path) = path else { return; };

                        let str_path = path.to_str().unwrap();
                        let transform = Transform::with_name(str_path);

                        let loaded_model = self.object_loader.load(str_path, &mut self.texture_loader);
                        match loaded_model {
                            Ok(_) => { self.current_scene.ecs.add_entity().with(transform).with::<ModelHandle>(str_path.into()); },
                            Err(_) => eprintln!("Failed to load model at \"{str_path}\""),
                        };
                    }
                };
            });
        });


        self.glow.paint(self.renderer.window.window());
    }

    fn handle_events(
        &mut self,
        event: Event<()>, 
        control_flow: &mut glutin::event_loop::ControlFlow,
    ) {
        match event {
            Event::WindowEvent { event, .. } => {

                // Close window
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) 
                    || self.input.just_pressed(VirtualKeyCode::Escape) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                // Resize window
                if let WindowEvent::Resized(physical_size) = &event {
                    self.current_scene.window_size_changed(physical_size);
                    self.renderer.window_resized(physical_size)

                } else if let WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    ..
                } = &event
                {
                    self.renderer.window_resized(*new_inner_size)
                }

                self.glow.on_event(&event);

                // Input event
                if let 
                    WindowEvent::KeyboardInput { .. } 
                | WindowEvent::CursorMoved { .. } 
                | WindowEvent::MouseInput { .. } 
                = event {
                    self.input.handle_events(&event);
                }


                self.renderer.window.window().request_redraw(); // TODO(emilk): ask egui if the events warrants a repaint instead
            }

            Event::LoopDestroyed => {
                self.glow.destroy();
            }

            _ => (),
        }
    }

    pub fn run(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {

        // Platform-dependent event handlers to workaround a winit bug
        // See: https://github.com/rust-windowing/winit/issues/987
        // See: https://github.com/rust-windowing/winit/issues/1619
        if let glutin::event::Event::MainEventsCleared = event {
            if !cfg!(windows) {

                // draw things behind egui here
                let current_frame = std::time::Instant::now();
                self.input.update((current_frame - self.last_frame).as_secs_f32());

                self.current_scene.camera.update(&mut self.input);


                self.current_scene.ecs.do_all_some::<ModelHandle, ()>(|(_id, model_handle)| {
                    self.object_loader.load(model_handle.name(), &mut self.texture_loader).unwrap_or_else(|e| {
                        eprintln!("Error: {e:?}");
                    });
                    None
                });


                self.renderer.render(
                    &self.current_scene.camera,
                    &mut self.current_scene.ecs,
                    &mut self.shader_loader,
                    &self.object_loader
                );

                self.app_ui();

                // draw things on top of egui here
                self.last_frame = current_frame;
                self.input.frame_end();

                self.renderer.window.swap_buffers().unwrap();
            }
        }

        self.handle_events(
            event,
            control_flow,
        );
    }

    pub fn get_resource_managers(&mut self) -> (&mut TextureLoader, &mut ObjLoader) {
        (&mut self.texture_loader, &mut self.object_loader)
    }
}
