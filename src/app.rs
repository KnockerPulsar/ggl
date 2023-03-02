use std::{sync::Arc, env};

use egui_gizmo::GizmoMode;
use glow::HasContext;
use glutin::{event::{WindowEvent, VirtualKeyCode, Event}, event_loop::ControlFlow};

use crate::{gl::{set_gl, get_gl}, scene::Scene, shader_loader::ShaderLoader, asset_loader::TextureLoader, obj_loader::{ObjLoader, ModelHandle, ModelType}, input::InputSystem, transform::Transform, light_system};

type GlutinWindow = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

pub struct App {
    window_width: i32,
    window_height: i32,
    window: GlutinWindow,

    glow: egui_glow::EguiGlow,

    current_scene: Option<Scene>, 

    shader_loader : ShaderLoader,
    texture_loader: TextureLoader,
    object_loader : ObjLoader,
    input: InputSystem,

    last_frame: std::time::Instant,
    lights_on: bool
}

impl App {
    pub fn init(
        window_width: usize, 
        window_height: usize,
    ) -> (App, glutin::event_loop::EventLoop<()>) {
        let window_width = window_width as i32;
        let window_height = window_height as i32;

        let (window, egui_glow, event_loop) = Self::init_window(window_width, window_height);

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


        let app = App {
            window_width,
            window_height,

            window,
            glow: egui_glow,

            current_scene: None,

            shader_loader,
            texture_loader,
            object_loader,       
            input,

            last_frame,
            lights_on: true 
        };

        (app, event_loop)
    }

    fn init_window(window_width: i32, window_height: i32) -> (GlutinWindow, egui_glow::EguiGlow, glutin::event_loop::EventLoop<()>) {
        let (gl, _, window, event_loop) = {
            let event_loop: glutin::event_loop::EventLoop<()> = glutin::event_loop::EventLoopBuilder::with_user_event().build();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("GG OpenGl")
                .with_inner_size(glutin::dpi::LogicalSize::new(window_width, window_height));

            unsafe {
                let window = glutin::ContextBuilder::new()
                    .with_depth_buffer(24)
                    .with_vsync(true)
                    .with_hardware_acceleration(Some(true))
                    .build_windowed(window_builder, &event_loop)
                    .unwrap()
                    .make_current()
                    .unwrap();

                let gl =
                    glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
                (gl, "#version 330", window, event_loop)
            }
        };


        let gl = set_gl(Arc::new(gl));
        let egui_glow = egui_glow::EguiGlow::new(&event_loop, Arc::clone(get_gl()));

        unsafe {
            gl.viewport(0, 0, window_width, window_height);
        }

        (window, egui_glow, event_loop)
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.current_scene = Some(scene)
    }

    fn egui_ui(&mut self) {

        let mut new_entity = false;

        self.glow.run(self.window.window(), |egui_ctx| {
            egui::Window::new("ggl").show(egui_ctx, |ui| {

                if let Some(current_scene) = &mut self.current_scene {
                    ui.spacing();
                    ui.heading("Entities");

                    ui.group(|ui| {
                        current_scene.entities_egui(ui);
                        if ui.button("New Empty Entity").clicked() {
                            new_entity = true;
                        }
                    });

                    current_scene.selected_entity_gizmo(egui_ctx);

                    if self.input.is_down(glutin::event::VirtualKeyCode::T) {
                        current_scene.gizmo_mode = GizmoMode::Translate;
                    }

                    if self.input.is_down(glutin::event::VirtualKeyCode::R) {
                        current_scene.gizmo_mode = GizmoMode::Rotate;
                    }

                    if self.input.is_down(glutin::event::VirtualKeyCode::Y) {
                        current_scene.gizmo_mode = GizmoMode::Scale;
                    }
                }

                ui.spacing();
                ui.heading("Global light toggle");

                ui.group(|ui| {
                    ui.checkbox(&mut self.lights_on, "Lights on?");
                });

                ui.spacing();
                ui.heading("Load models");

                ui.group(|ui| {
                    if ui.button("Load").clicked() {

                        if self.current_scene.is_none() {
                            self.current_scene = Some(Scene::empty(self.window_width, self.window_height));
                            println!("Created new empty scene");
                        }

                        if let Some(current_scene) = &mut self.current_scene {
                            let path = rfd::FileDialog::new().add_filter("Object model", &["obj"]).pick_file(); 
                            if let Some(path) = path {
                                let str_path = path.to_str().unwrap();
                                let transform = Transform::with_name(str_path);

                                current_scene.ecs
                                    .add_entity()
                                    .with(transform)
                                    .with(self.object_loader.load(str_path, &mut self.texture_loader).unwrap());
                            }
                        }
                    }
                });
            });
        });

        if new_entity {
            if let Some(scene) = &mut self.current_scene {
                let num_entities = scene.ecs.num_entities();
                scene.ecs.add_entity().with(Transform::with_name(format!("Entity {num_entities}")));
            }
        }

        self.glow.paint(self.window.window());
    }

    fn handle_events(
        &mut self,
        event: glutin::event::Event<()>, 
        control_flow: &mut glutin::event_loop::ControlFlow,
    ) {
        let gl = get_gl();

        match event {
            glutin::event::Event::WindowEvent { event, .. } => {

                // Close window
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) 
                    || self.input.just_pressed(VirtualKeyCode::Escape) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                // Resize window
                if let glutin::event::WindowEvent::Resized(physical_size) = &event {
                    self.window.resize(*physical_size);

                    if let Some(current_scene) = &mut self.current_scene {
                        current_scene.window_size_changed(physical_size);
                    }

                    unsafe {
                        gl.viewport(0, 0, self.window_width, self.window_height);
                    }
                } else if let glutin::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    ..
                } = &event
                {
                    self.window.resize(**new_inner_size);
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


                self.window.window().request_redraw(); // TODO(emilk): ask egui if the events warrants a repaint instead
            }

            glutin::event::Event::LoopDestroyed => {
                self.glow.destroy();
            }

            _ => (),
        }
    }

    pub fn run(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        let gl = get_gl();

        // Platform-dependent event handlers to workaround a winit bug
        // See: https://github.com/rust-windowing/winit/issues/987
        // See: https://github.com/rust-windowing/winit/issues/1619
        if let glutin::event::Event::MainEventsCleared = event {
            if !cfg!(windows) {
                unsafe{
                    gl.enable(glow::DEPTH_TEST);
                    gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                    gl.enable(glow::BLEND);
                    gl.clear_color(0.2, 0.2, 0.2, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }

                // draw things behind egui here
                let current_frame = std::time::Instant::now();
                self.input.update((current_frame - self.last_frame).as_secs_f32());

                if let Some(current_scene) = &mut self.current_scene {
                    current_scene.camera.update(&mut self.input);
                }

                self.render_system();

                self.egui_ui();

                // draw things on top of egui here
                self.last_frame = current_frame;
                self.input.frame_end();

                self.window.swap_buffers().unwrap();

            }
        }

        self.handle_events(
            event,
            control_flow,
        );
    }

    fn render_system(&mut self) {
        if let Some(current_scene) = &mut self.current_scene {
            let view = current_scene.camera.get_view_matrix();

            let lit_shader = self.shader_loader.borrow_shader("default").unwrap();
            lit_shader.use_program();
            light_system(&mut current_scene.ecs, lit_shader, &mut self.lights_on);
            lit_shader
                .set_vec3("u_view_pos", current_scene.camera.get_pos())
                .set_float("u_material.shininess", 32.0)
                .set_mat4("projection", current_scene.camera.get_proj_matrix())
                .set_mat4("view", view);

            // lit_shader.set_float("u_material.emissive_factor", 1.0);

            current_scene.ecs.do_all::<ModelHandle, Transform>(|model_handle, transform| {
                let model = self.object_loader.borrow(model_handle).unwrap();

                match model.model_type {
                    ModelType::Normal => {
                        model.draw_normal(&mut self.shader_loader, transform)
                    },
                    ModelType::Billboard => model.draw_billboard(&mut self.shader_loader, transform, &current_scene.camera),
                };
                
            });
        }
    }

    pub fn get_resource_managers(&mut self) -> (&mut TextureLoader, &mut ObjLoader) {
        (&mut self.texture_loader, &mut self.object_loader)
    }
}
