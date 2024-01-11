use std::{env, sync::Arc};

use glow::HasContext;
use glutin::{event::*, event_loop::ControlFlow};

use crate::gl::set_gl;
use crate::loaders::utils::Handle;

use crate::renderer::GlutinWindow;
use crate::{add_default_component, ecs::AddableComponent, light::*, renderer::Renderer, ui::*};

use crate::{gl::get_gl, input::InputSystem, loaders::*, scene::Scene, transform::Transform};

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

    shader_loader: ShaderLoader,
    texture_loader: TextureLoader,
    object_loader: ObjLoader,
    input: InputSystem,

    last_frame: std::time::Instant,
    cumulative_time: std::time::Instant,
    current_panel: Panels,
}

impl App {
    fn init_window(window_width: i32, window_height: i32, event_loop: &EventLoop) -> GlutinWindow {
        let window = {
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("GG OpenGl")
                .with_inner_size(glutin::dpi::LogicalSize::new(window_width, window_height));

            unsafe {
                let window = glutin::ContextBuilder::new()
                    .with_depth_buffer(24)
                    // .with_vsync(true)
                    .with_hardware_acceleration(Some(true))
                    .build_windowed(window_builder, event_loop)
                    .unwrap()
                    .make_current()
                    .unwrap();

                window
            }
        };

        window
    }

    fn init_opengl(window: &GlutinWindow, width: i32, height: i32) {
        unsafe {
            let gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            let gl = set_gl(Arc::new(gl));
            gl.viewport(0, 0, width, height);
        }
    }

    pub fn init(
        window_width: usize,
        window_height: usize,
    ) -> (App, glutin::event_loop::EventLoop<()>) {
        let cumulative_time = std::time::Instant::now();
        let event_loop: EventLoop = glutin::event_loop::EventLoopBuilder::with_user_event().build();
        let window = Self::init_window(window_width as i32, window_height as i32, &event_loop);

        Self::init_opengl(&window, window_width as i32, window_height as i32);
        let egui_glow = egui_glow::EguiGlow::new(&event_loop, Arc::clone(get_gl()));

        println!(
            "Current working directory: {}",
            env::current_dir().unwrap().to_str().unwrap()
        );

        println!("Loading, please wait...");

        let mut shader_loader = ShaderLoader::new(&[]);
        let mut texture_loader = TextureLoader::new();
        let object_loader = ObjLoader::new(&mut shader_loader, &mut texture_loader);

        let last_frame = std::time::Instant::now();
        let input = InputSystem::new();

        let mut renderer = Renderer::new(window, window_width, window_height, &mut shader_loader);

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
            current_panel: Panels::Entities,
            cumulative_time,
        };

        (app, event_loop)
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.current_scene = scene
    }

    fn app_ui(&mut self) {
        self.glow.run(self.renderer.window.window(), |egui_ctx| {
            egui::Window::new("ggl")
                .hscroll(false)
                .vscroll(false)
                .default_width(400.0)
                .default_height(250.0)
                .show(egui_ctx, |ui| {
                    egui::TopBottomPanel::top("")
                        .resizable(false)
                        .default_height(35.0)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.selectable_value(
                                    &mut self.current_panel,
                                    Panels::Entities,
                                    "Entities",
                                );
                                ui.selectable_value(
                                    &mut self.current_panel,
                                    Panels::Models,
                                    "Models",
                                );
                            });
                        });

                    selected_entity_gizmo(egui_ctx, &mut self.current_scene, &self.input);

                    match self.current_panel {
                        Panels::Entities => {
                            let (selected_entity, add_entity, add_component) = entities_panel(
                                ui,
                                &mut self.current_scene,
                                &mut self.renderer.lights_on,
                            );

                            self.current_scene.selected_entity = selected_entity;

                            if add_entity {
                                self.current_scene.ecs.add_empty_entity();
                            }

                            if let Some(selected_entity) = self.current_scene.selected_entity {
                                if add_component {
                                    add_default_component!(
                                        &mut self.current_scene.ecs,
                                        selected_entity,
                                        [Transform, PointLight, SpotLight, DirectionalLight]
                                    );
                                }
                            }
                        }
                        Panels::Models => {
                            let path = models_panel(ui, &mut self.object_loader);

                            let Some(path) = path else {
                                return;
                            };

                            let str_path = path.to_str().unwrap();
                            let transform = Transform::with_name(str_path);

                            let model_name = format!("Model {}", self.object_loader.models().len());
                            let loaded_model = self.object_loader.load_model(
                                model_name,
                                str_path,
                                &mut self.texture_loader,
                                &mut self.shader_loader,
                            );

                            match loaded_model {
                                Ok(model_rc) => {
                                    self.current_scene
                                        .ecs
                                        .add_entity()
                                        .with(transform)
                                        .with(Handle::clone(&model_rc));
                                }
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
                    || self.input.just_pressed(VirtualKeyCode::Escape)
                {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                // Resize window
                if let WindowEvent::Resized(physical_size) = &event {
                    self.current_scene.window_size_changed(physical_size);
                    self.renderer.window_resized(physical_size)
                } else if let WindowEvent::ScaleFactorChanged { new_inner_size, .. } = &event {
                    self.renderer.window_resized(new_inner_size)
                }

                self.glow.on_event(&event);

                // Input event
                if let WindowEvent::KeyboardInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. } = event
                {
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
                self.input
                    .update((current_frame - self.last_frame).as_secs_f32());

                self.current_scene.camera.update(&mut self.input);

                self.renderer.render(
                    &mut self.current_scene.camera,
                    &mut self.current_scene.ecs,
                    &mut self.shader_loader,
                    (current_frame - self.cumulative_time).as_secs_f32(),
                );

                self.app_ui();

                // draw things on top of egui here
                self.last_frame = current_frame;
                self.input.frame_end();

                self.renderer.window.swap_buffers().unwrap();
            }
        }

        self.handle_events(event, control_flow);
    }

    pub fn get_resource_managers(
        &mut self,
    ) -> (&mut TextureLoader, &mut ObjLoader, &mut ShaderLoader) {
        (
            &mut self.texture_loader,
            &mut self.object_loader,
            &mut self.shader_loader,
        )
    }
}

// TODO: Adding models to entities
// TODO: Material list
// TODO: Texture list
